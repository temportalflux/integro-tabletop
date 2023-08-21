use super::id::IdPath;
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, ValueExt},
	utility::NotInList,
};
use anyhow::Context;
use enumset::{EnumSet, EnumSetType};
use std::{
	collections::BTreeSet,
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
};

#[derive(Clone)]
pub enum Value<Context, T> {
	Specific(T),
	Options {
		id: IdPath,
		amount: crate::utility::Value<Context, i32>,
		options: BTreeSet<T>,
		is_applicable: Option<Arc<dyn Fn(&T, &Context) -> bool + 'static + Send + Sync>>,
	},
}

impl<Context, T> std::fmt::Debug for Value<Context, T>
where
	T: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Specific(arg0) => f.debug_tuple("Specific").field(arg0).finish(),
			Self::Options {
				id,
				amount,
				options,
				is_applicable,
			} => f
				.debug_struct("Options")
				.field("id", id)
				.field("amount", amount)
				.field("options", options)
				.field("has_is_applicable", &is_applicable.is_some())
				.finish(),
		}
	}
}

impl<Context, T> PartialEq for Value<Context, T>
where
	Context: 'static,
	T: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Specific(l0), Self::Specific(r0)) => l0 == r0,
			(
				Self::Options {
					id: l_id,
					amount: l_amount,
					options: l_options,
					is_applicable: _,
				},
				Self::Options {
					id: r_id,
					amount: r_amount,
					options: r_options,
					is_applicable: _,
				},
			) => l_id == r_id && l_amount == r_amount && l_options == r_options,
			_ => false,
		}
	}
}

impl<Context, T> Value<Context, T>
where
	T: ToString + FromStr,
{
	fn id_path(&self) -> Option<&IdPath> {
		match self {
			Self::Specific(_) => None,
			Self::Options { id, .. } => Some(id),
		}
	}

	pub fn set_data_path(&self, parent: &Path) {
		if let Some(id_path) = self.id_path() {
			id_path.set_path(parent);
		}
	}

	pub fn get_data_path(&self) -> Option<PathBuf> {
		self.id_path().map(|id_path| id_path.as_path()).flatten()
	}

	pub fn add_default_options(&mut self)
	where
		T: EnumSetType + Ord,
	{
		let Self::Options{ options, .. } = self else { return; };
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
		let Self::Options{ is_applicable, .. } = self else { return; };
		*is_applicable = Some(Arc::new(callback));
	}
}

impl<Context, T> FromKDL for Value<Context, T>
where
	T: ToString + FromStr + Ord,
	T::Err: std::error::Error + Send + Sync + 'static,
	crate::utility::Value<Context, i32>: FromKDL,
{
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

				let mut options = BTreeSet::new();
				for str in node.query_str_all("scope() > option", 0)? {
					options.insert(T::from_str(str)?);
				}

				Ok(Self::Options {
					id,
					amount,
					options,
					is_applicable: None,
				})
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
			Self::Options {
				id,
				amount,
				options,
				..
			} => {
				let mut node = node.with_entry("Any");
				if let Some(id) = id.get_id() {
					node.push_entry(("id", id.clone()));
				}
				if amount != &crate::utility::Value::Fixed(1) {
					node.push_child_t("amount", amount);
				}
				for option in options {
					node.push_child_t("option", option);
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
		let Self::Options { id, amount, options, is_applicable } = self else { return Ok(None); };
		let Some(data_path) = id.as_path() else { return Err(super::InvalidDataPath); };
		let mut valid_options = BTreeSet::new();
		for option in options {
			if let Some(is_applicable) = is_applicable.as_ref() {
				if !(**is_applicable)(option, context) {
					continue;
				}
			}
			valid_options.insert(option.to_string());
		}
		Ok(Some(super::DataOption {
			data_path,
			name: name.into(),
			kind: super::Kind::StringEntry {
				amount: amount.evaluate(context) as usize,
				options: valid_options,
			},
		}))
	}
}
