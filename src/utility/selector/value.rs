use super::id::IdPath;
use crate::{kdl_ext::NodeContext, system::mutator::ReferencePath, utility::NotInList};
use anyhow::Context;
use enumset::{EnumSet, EnumSetType};
use kdlize::{
	ext::{DocumentExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};
use std::{collections::BTreeSet, path::PathBuf, str::FromStr, sync::Arc};

#[derive(Clone, PartialEq, Debug)]
pub enum Value<Context: 'static, T> {
	Specific(T),
	Options(ValueOptions<Context, T>),
}

#[derive(Clone)]
pub struct ValueOptions<Context, T> {
	pub id: IdPath,
	pub amount: crate::utility::Value<Context, i32>,
	pub options: BTreeSet<T>,
	pub cannot_match: Vec<IdPath>,
	pub is_applicable: Option<Arc<dyn Fn(&T, &Context) -> bool + 'static + Send + Sync>>,
}

impl<Context, T> Default for ValueOptions<Context, T> {
	fn default() -> Self {
		Self {
			id: Default::default(),
			amount: crate::utility::Value::Fixed(1),
			options: BTreeSet::default(),
			cannot_match: [].into(),
			is_applicable: None,
		}
	}
}

impl<Context, T> std::fmt::Debug for ValueOptions<Context, T>
where
	T: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ValueOptions")
			.field("id", &self.id)
			.field("amount", &self.amount)
			.field("options", &self.options)
			.field("has_is_applicable", &self.is_applicable.is_some())
			.finish()
	}
}

impl<Context, T> PartialEq for ValueOptions<Context, T>
where
	Context: 'static,
	T: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id && self.amount == other.amount && self.options == other.options
	}
}

impl<Context, T> Value<Context, T>
where
	T: ToString + FromStr,
{
	fn id_path(&self) -> Option<&IdPath> {
		match self {
			Self::Specific(_) => None,
			Self::Options(ValueOptions { id, .. }) => Some(id),
		}
	}

	pub fn set_data_path(&self, parent: &ReferencePath) {
		if let Some(id_path) = self.id_path() {
			id_path.set_path(&parent);
		}
		if let Self::Options(ValueOptions { cannot_match, .. }) = self {
			for id_path in cannot_match {
				id_path.set_path(&parent);
			}
		}
	}

	pub fn get_data_path(&self) -> Option<PathBuf> {
		self.id_path().map(|id_path| id_path.data()).flatten()
	}

	pub fn add_default_options(&mut self)
	where
		T: EnumSetType + Ord,
	{
		let Self::Options(ValueOptions { options, .. }) = self else {
			return;
		};
		if options.is_empty() {
			*options = EnumSet::all().into_iter().collect();
		}
	}

	pub fn with_default_options(&self) -> Self
	where
		Context: Clone,
		T: EnumSetType + Ord,
	{
		let mut out = self.clone();
		out.add_default_options();
		out
	}

	pub fn set_is_applicable<F>(&mut self, callback: F)
	where
		F: Fn(&T, &Context) -> bool + 'static + Send + Sync,
	{
		let Self::Options(ValueOptions { is_applicable, .. }) = self else {
			return;
		};
		*is_applicable = Some(Arc::new(callback));
	}
}

impl<Context, T> FromKdl<NodeContext> for Value<Context, T>
where
	T: ToString + FromStr + Ord,
	T::Err: std::error::Error + Send + Sync + 'static,
	crate::utility::Value<Context, i32>: FromKdl<NodeContext>,
	anyhow::Error: From<<crate::utility::Value<Context, i32> as FromKdl<NodeContext>>::Error>,
{
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		let key = entry
			.as_str_req()
			.context("Selector keys must be a string with the selector name")?;
		match key {
			"Specific" => {
				let entry = node.next_req()?;
				Ok(Self::Specific(T::from_str(entry.value().as_str_req()?)?))
			}
			"Any" | "AnyOf" => {
				let id = node.get_str_opt("id")?.into();

				let amount = match node.query_opt_t("scope() > amount")? {
					None => crate::utility::Value::Fixed(1),
					Some(eval) => eval,
				};

				let mut cannot_match = Vec::new();
				for str in node.query_str_all("scope() > cannot_match", 0)? {
					cannot_match.push(str.into());
				}

				let mut options = BTreeSet::new();
				for str in node.query_str_all("scope() > option", 0)? {
					options.insert(T::from_str(str)?);
				}

				Ok(Self::Options(ValueOptions {
					id,
					amount,
					options,
					cannot_match,
					is_applicable: None,
				}))
			}
			name => Err(NotInList(name.into(), vec!["Specific", "Any", "AnyOf"]).into()),
		}
	}
}
impl<Context, T> AsKdl for Value<Context, T>
where
	Context: 'static,
	T: ToString + FromStr + AsKdl,
	crate::utility::Value<Context, i32>: AsKdl,
{
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Specific(value) => node.with_entry("Specific").with_extension(value.as_kdl()),
			Self::Options(ValueOptions {
				id,
				amount,
				options,
				cannot_match,
				..
			}) => {
				let mut node = node.with_entry("Any");
				if let Some(id) = id.get_id() {
					node.push_entry(("id", id.into_owned()));
				}
				if amount != &crate::utility::Value::Fixed(1) {
					node.push_child_t(("amount", amount));
				}
				for id_path in cannot_match {
					let Some(id_str) = id_path.get_id() else {
						continue;
					};
					node.push_child_entry("cannot_match", id_str.into_owned());
				}
				for option in options {
					node.push_child_t(("option", option));
				}
				node
			}
		}
	}
}

impl<Context, T> Value<Context, T>
where
	Context: 'static + Send + Sync,
	T: 'static + Clone + Send + Sync + std::fmt::Debug + PartialEq + ToString,
{
	pub fn as_data(
		&self,
		name: impl Into<String>,
		context: &Context,
	) -> Result<Option<super::DataOption>, super::InvalidDataPath> {
		let Self::Options(ValueOptions {
			id,
			amount,
			options,
			cannot_match,
			is_applicable,
		}) = self
		else {
			return Ok(None);
		};
		let Some(data_path) = id.data() else {
			return Err(super::InvalidDataPath);
		};
		let mut blocked_options = BTreeSet::new();
		for option in options {
			if let Some(is_applicable) = is_applicable.as_ref() {
				if !(**is_applicable)(option, context) {
					blocked_options.insert(option.to_string());
				}
			}
		}
		let cannot_match = cannot_match.iter().filter_map(super::IdPath::data).collect();
		Ok(Some(super::DataOption {
			data_path,
			name: name.into(),
			kind: super::Kind::StringEntry {
				amount: amount.evaluate(context) as usize,
				options: options.iter().map(T::to_string).collect(),
				blocked_options,
				cannot_match,
			},
		}))
	}
}
