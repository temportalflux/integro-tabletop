use crate::{
	kdl_ext::{DocumentExt, NodeContext, NodeExt, ValueExt},
	GeneralError,
};
use anyhow::Context;
use derivative::Derivative;
use enumset::{EnumSet, EnumSetType};
use std::{
	path::{Path, PathBuf},
	str::FromStr,
	sync::{Arc, RwLock},
};

#[derive(Clone, Default, Derivative)]
#[derivative(PartialEq)]
pub struct IdPath {
	id: Option<String>,
	#[derivative(PartialEq = "ignore")]
	absolute_path: Arc<RwLock<PathBuf>>,
}
impl<T: Into<String>> From<Option<T>> for IdPath {
	fn from(value: Option<T>) -> Self {
		Self {
			id: value.map(|t| t.into()),
			absolute_path: Arc::new(RwLock::new(PathBuf::new())),
		}
	}
}
impl From<&str> for IdPath {
	fn from(value: &str) -> Self {
		Self::from(Some(value))
	}
}
impl IdPath {
	fn set_path(&self, path: &Path) {
		let path = match &self.id {
			Some(id) => path.join(id),
			None => path.to_owned(),
		};
		let path = PathBuf::from(path.to_str().unwrap().replace("\\", "/"));
		*self.absolute_path.write().unwrap() = path;
	}

	fn as_path(&self) -> PathBuf {
		self.absolute_path.read().unwrap().clone()
	}
}
impl std::fmt::Debug for IdPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"IdPath(id={:?}, path={:?})",
			self.id,
			*self.absolute_path.read().unwrap()
		)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Selector<T: ToString + FromStr> {
	Specific(T),
	AnyOf {
		id: IdPath,
		cannot_match: Vec<IdPath>,
		options: Vec<T>,
	},
	Any {
		id: IdPath,
		cannot_match: Vec<IdPath>,
	},
}

impl<T> Selector<T>
where
	T: ToString + FromStr,
{
	fn id_path(&self) -> Option<&IdPath> {
		match self {
			Self::Specific(_) => None,
			Self::AnyOf { id, .. } => Some(id),
			Self::Any { id, .. } => Some(id),
		}
	}

	pub fn set_data_path(&self, parent: &Path) {
		if let Some(id_path) = self.id_path() {
			id_path.set_path(parent);
		}
		match &self {
			Self::Any { cannot_match, .. } | Self::AnyOf { cannot_match, .. } => {
				for id_path in cannot_match {
					id_path.set_path(parent);
				}
			}
			Self::Specific(_) => {}
		}
	}

	pub fn get_data_path(&self) -> Option<PathBuf> {
		let path = self.id_path().map(|id_path| id_path.as_path());
		if let Some(path) = &path {
			if path.to_str() == Some("") {
				log::warn!(target: "utility", "Selector data path is empty, <MutatorGroup/Mutator/Selector>::set_data_path was not called somewhere.");
			}
		}
		path
	}

	pub fn from_kdl(
		node: &kdl::KdlNode,
		entry: &kdl::KdlEntry,
		ctx: &mut NodeContext,
		map_value: impl Fn(&kdl::KdlValue) -> anyhow::Result<T>,
	) -> anyhow::Result<Self> {
		let key = entry
			.as_str_req()
			.context("Selector keys must be a string with the selector name")?;
		match key {
			"Specific" => {
				let idx = ctx.consume_idx();
				let value = node
					.get(idx)
					.ok_or(crate::kdl_ext::EntryMissing(node.clone(), idx.into()))?;
				Ok(Self::Specific(map_value(value)?))
			}
			"AnyOf" => {
				let id = node.get_str_opt("id")?.into();

				let cannot_match = node.query_str_all("scope() > cannot-match", 0)?;
				let cannot_match = cannot_match.into_iter().map(IdPath::from).collect();

				let mut options = Vec::new();
				for kdl_value in node.query_get_all("scope() > option", 0)? {
					options.push(map_value(kdl_value)?);
				}
				Ok(Self::AnyOf {
					id,
					options,
					cannot_match,
				})
			}
			"Any" => {
				let id = node.get_str_opt("id")?.into();
				let cannot_match = node.query_str_all("scope() > cannot-match", 0)?;
				let cannot_match = cannot_match.into_iter().map(IdPath::from).collect();
				Ok(Self::Any { id, cannot_match })
			}
			_ => Err(GeneralError(format!(
				"Invalid selector key {key:?}, expected Specific, Any, or AnyOf."
			))
			.into()),
		}
	}
}

impl Selector<String> {
	pub fn as_meta_str(&self, name: impl Into<String>) -> Option<SelectorMeta> {
		SelectorMeta::from_string(name, &self)
	}
}

impl<T> Selector<T>
where
	T: 'static + ToString + FromStr + EnumSetType,
{
	pub fn as_meta_enum(&self, name: impl Into<String>) -> Option<SelectorMeta> {
		SelectorMeta::from_enum(name, &self)
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct SelectorMeta {
	pub data_path: PathBuf,
	pub name: String,
	pub options: SelectorOptions,
}
impl SelectorMeta {
	fn from_string(name: impl Into<String>, selector: &Selector<String>) -> Option<Self> {
		let Some(data_path) = selector.get_data_path() else { return None; };
		let Some(options) = SelectorOptions::from_string(selector) else { return None; };
		Some(Self {
			name: name.into(),
			data_path,
			options,
		})
	}

	fn from_enum<T>(name: impl Into<String>, selector: &Selector<T>) -> Option<Self>
	where
		T: 'static + ToString + FromStr + EnumSetType,
	{
		let Some(data_path) = selector.get_data_path() else { return None; };
		let Some(options) = SelectorOptions::from_enum(selector) else { return None; };
		Some(Self {
			name: name.into(),
			data_path,
			options,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum SelectorOptions {
	/// User can provide any string value
	Any,
	/// User must select one of these string values
	AnyOf {
		/// The valid string values.
		options: Vec<String>,
		/// A list of other selectors that this selector cannot have the same value as.
		cannot_match: Option<Vec<PathBuf>>,
	},
}

impl SelectorOptions {
	pub fn from_string(selector: &Selector<String>) -> Option<Self> {
		match selector {
			Selector::Specific(_) => None,
			Selector::AnyOf {
				cannot_match,
				options,
				..
			} => {
				let cannot_match = (!cannot_match.is_empty())
					.then(|| cannot_match.iter().map(IdPath::as_path).collect());
				Some(Self::AnyOf {
					options: options.clone(),
					cannot_match,
				})
			}
			Selector::Any { .. } => Some(Self::Any),
		}
	}

	fn iter_to_str<U>(iter: impl Iterator<Item = U>) -> Vec<String>
	where
		U: ToString,
	{
		iter.map(|v| v.to_string()).collect::<Vec<_>>()
	}

	pub fn from_enum<T>(selector: &Selector<T>) -> Option<Self>
	where
		T: 'static + ToString + FromStr + EnumSetType,
	{
		match selector {
			Selector::Specific(_) => None,
			Selector::AnyOf {
				options,
				cannot_match,
				..
			} => {
				let options = options.iter().map(|t| *t);
				let cannot_match = (!cannot_match.is_empty())
					.then(|| cannot_match.iter().map(IdPath::as_path).collect());
				Some(Self::AnyOf {
					options: Self::iter_to_str(options),
					cannot_match,
				})
			}
			Selector::Any { cannot_match, .. } => {
				let options = EnumSet::<T>::all().into_iter();
				let cannot_match = (!cannot_match.is_empty())
					.then(|| cannot_match.iter().map(IdPath::as_path).collect());
				Some(Self::AnyOf {
					options: Self::iter_to_str(options),
					cannot_match,
				})
			}
		}
	}
}

#[derive(Default)]
pub struct SelectorMetaVec(Vec<SelectorMeta>);
impl SelectorMetaVec {
	pub fn with_str(mut self, name: impl Into<String>, selector: &Selector<String>) -> Self {
		if let Some(meta) = SelectorMeta::from_string(name, selector) {
			self.0.push(meta);
		}
		self
	}

	pub fn with_enum<T>(mut self, name: impl Into<String>, selector: &Selector<T>) -> Self
	where
		T: 'static + ToString + FromStr + EnumSetType,
	{
		if let Some(meta) = SelectorMeta::from_enum(name, selector) {
			self.0.push(meta);
		}
		self
	}

	pub fn to_vec(self) -> Option<Vec<SelectorMeta>> {
		(!self.0.is_empty()).then_some(self.0)
	}
}
