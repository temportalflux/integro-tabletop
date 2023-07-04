use super::NotInList;
use crate::kdl_ext::{AsKdl, DocumentExt, NodeBuilder, ValueExt};
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
	pub fn get_id(&self) -> Option<&String> {
		self.id.as_ref()
	}

	pub fn set_path(&self, path: &Path) {
		let path = match &self.id {
			Some(id) => path.join(id),
			None => path.to_owned(),
		};
		let path = PathBuf::from(path.to_str().unwrap().replace("\\", "/"));
		*self.absolute_path.write().unwrap() = path;
	}

	pub fn as_path(&self) -> Option<PathBuf> {
		let path = self.absolute_path.read().unwrap().clone();
		if path.to_str() == Some("") {
			return None;
		}
		Some(path)
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
		amount: usize,
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
		self.id_path().map(|id_path| id_path.as_path()).flatten()
	}
}
impl<T> crate::kdl_ext::FromKDL for Selector<T>
where
	T: ToString + FromStr,
	T::Err: std::error::Error + Send + Sync + 'static,
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
			"AnyOf" => {
				let id = node.get_str_opt("id")?.into();

				let cannot_match = node.query_str_all("scope() > cannot-match", 0)?;
				let cannot_match = cannot_match.into_iter().map(IdPath::from).collect();

				let mut options = Vec::new();
				for kdl_value in node.query_get_all("scope() > option", 0)? {
					options.push(T::from_str(kdl_value.as_str_req()?)?);
				}
				Ok(Self::AnyOf {
					id,
					options,
					amount: 1,
					cannot_match,
				})
			}
			"Any" => {
				let id = node.get_str_opt("id")?.into();
				let cannot_match = node.query_str_all("scope() > cannot-match", 0)?;
				let cannot_match = cannot_match.into_iter().map(IdPath::from).collect();
				Ok(Self::Any { id, cannot_match })
			}
			name => Err(NotInList(name.into(), vec!["Specific", "Any", "AnyOf"]).into()),
		}
	}
}
impl<T: ToString + FromStr + AsKdl> AsKdl for Selector<T> {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Specific(value) => node.with_entry("Specific").with_extension(value.as_kdl()),
			Self::AnyOf {
				id,
				cannot_match,
				amount: _,
				options,
			} => {
				let mut node = node.with_entry("AnyOf");
				if let Some(id) = id.get_id() {
					node.push_entry(("id", id.clone()));
				}
				for cannot_match in cannot_match {
					let Some(id) = cannot_match.get_id() else { continue; };
					node.push_child_entry("cannot-match", id.clone());
				}
				for option in options {
					node.push_child_t("option", option);
				}
				node
			}
			Self::Any { id, cannot_match } => {
				let mut node = node.with_entry("Any");
				if let Some(id) = id.get_id() {
					node.push_entry(("id", id.clone()));
				}
				for cannot_match in cannot_match {
					let Some(id) = cannot_match.get_id() else { continue; };
					node.push_child_entry("cannot-match", id.clone());
				}
				node
			}
		}
	}
}

impl Selector<String> {
	pub fn as_meta_str(
		&self,
		name: impl Into<String>,
	) -> Result<Option<SelectorMeta>, InvalidDataPath> {
		SelectorMeta::from_string(name, &self)
	}
}

impl<T> Selector<T>
where
	T: 'static + ToString + FromStr + EnumSetType,
{
	pub fn as_meta_enum(
		&self,
		name: impl Into<String>,
	) -> Result<Option<SelectorMeta>, InvalidDataPath> {
		SelectorMeta::from_enum(name, &self)
	}
}

/// Allows the user to select an object in the database by its `SourceId`,
/// as long as the entry passes some criteria.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectSelector {
	id: IdPath,
	category: String,
	count: usize,
	criteria: Option<crate::database::app::Criteria>,
}
impl ObjectSelector {
	pub fn new(category: impl Into<String>, count: usize) -> Self {
		Self {
			id: IdPath::default(),
			category: category.into(),
			count,
			criteria: None,
		}
	}

	pub fn set_data_path(&self, parent: &Path) {
		self.id.set_path(parent);
	}

	pub fn get_data_path(&self) -> Option<PathBuf> {
		self.id.as_path()
	}

	pub fn count(&self) -> usize {
		self.count
	}

	pub fn set_criteria(&mut self, criteria: crate::database::app::Criteria) {
		self.criteria = Some(criteria);
	}

	pub fn criteria(&self) -> Option<&crate::database::app::Criteria> {
		self.criteria.as_ref()
	}
}

#[derive(Clone, PartialEq, thiserror::Error, Debug)]
#[error("Invalid selector data path")]
pub struct InvalidDataPath;

#[derive(Clone, PartialEq, Debug)]
pub struct SelectorMeta {
	pub data_path: PathBuf,
	pub name: String,
	pub options: SelectorOptions,
}
impl SelectorMeta {
	fn from_string(
		name: impl Into<String>,
		selector: &Selector<String>,
	) -> Result<Option<Self>, InvalidDataPath> {
		let Some(options) = SelectorOptions::from_string(selector) else { return Ok(None); };
		let Some(data_path) = selector.get_data_path() else { return Err(InvalidDataPath); };
		Ok(Some(Self {
			name: name.into(),
			data_path,
			options,
		}))
	}

	fn from_enum<T>(
		name: impl Into<String>,
		selector: &Selector<T>,
	) -> Result<Option<Self>, InvalidDataPath>
	where
		T: 'static + ToString + FromStr + EnumSetType,
	{
		let Some(options) = SelectorOptions::from_enum(selector) else { return Ok(None); };
		let Some(data_path) = selector.get_data_path() else { return Err(InvalidDataPath); };
		Ok(Some(Self {
			name: name.into(),
			data_path,
			options,
		}))
	}

	fn from_object(
		name: impl Into<String>,
		selector: &ObjectSelector,
	) -> Result<Option<Self>, InvalidDataPath> {
		let Some(options) = SelectorOptions::from_object(selector) else { return Ok(None); };
		let Some(data_path) = selector.get_data_path() else { return Err(InvalidDataPath); };
		Ok(Some(Self {
			name: name.into(),
			data_path,
			options,
		}))
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
		/// The number of options that can be selected.
		amount: usize,
		/// A list of other selectors that this selector cannot have the same value as.
		cannot_match: Option<Vec<PathBuf>>,
	},
	Object {
		count: usize,
		category: String,
		criteria: Option<crate::database::app::Criteria>,
	},
}

impl SelectorOptions {
	pub fn from_string(selector: &Selector<String>) -> Option<Self> {
		match selector {
			Selector::Specific(_) => None,
			Selector::AnyOf {
				cannot_match,
				options,
				amount,
				id: _,
			} => {
				let cannot_match = (!cannot_match.is_empty())
					.then(|| cannot_match.iter().filter_map(IdPath::as_path).collect());
				Some(Self::AnyOf {
					options: options.clone(),
					amount: *amount,
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
				amount,
				id: _,
			} => {
				let options = options.iter().map(|t| *t);
				let cannot_match = (!cannot_match.is_empty())
					.then(|| cannot_match.iter().filter_map(IdPath::as_path).collect());
				Some(Self::AnyOf {
					options: Self::iter_to_str(options),
					amount: *amount,
					cannot_match,
				})
			}
			Selector::Any {
				cannot_match,
				id: _,
			} => {
				let options = EnumSet::<T>::all().into_iter();
				let cannot_match = (!cannot_match.is_empty())
					.then(|| cannot_match.iter().filter_map(IdPath::as_path).collect());
				Some(Self::AnyOf {
					options: Self::iter_to_str(options),
					amount: 1,
					cannot_match,
				})
			}
		}
	}

	pub fn from_object(selector: &ObjectSelector) -> Option<Self> {
		Some(Self::Object {
			count: selector.count(),
			category: selector.category.clone(),
			criteria: selector.criteria.clone(),
		})
	}
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SelectorMetaVec(Vec<SelectorMeta>, Vec<InvalidDataPath>);
impl SelectorMetaVec {
	pub fn with_str(mut self, name: impl Into<String>, selector: &Selector<String>) -> Self {
		match SelectorMeta::from_string(name, selector) {
			Ok(Some(meta)) => {
				self.0.push(meta);
			}
			Err(err) => {
				self.1.push(err);
			}
			_ => {}
		}
		self
	}

	pub fn with_enum<T>(mut self, name: impl Into<String>, selector: &Selector<T>) -> Self
	where
		T: 'static + ToString + FromStr + EnumSetType,
	{
		match SelectorMeta::from_enum(name, selector) {
			Ok(Some(meta)) => {
				self.0.push(meta);
			}
			Err(err) => {
				self.1.push(err);
			}
			_ => {}
		}
		self
	}

	pub fn with_object(mut self, name: impl Into<String>, selector: &ObjectSelector) -> Self {
		match SelectorMeta::from_object(name, selector) {
			Ok(Some(meta)) => {
				self.0.push(meta);
			}
			Err(err) => {
				self.1.push(err);
			}
			_ => {}
		}
		self
	}

	pub fn as_vec(&self) -> &Vec<SelectorMeta> {
		&self.0
	}

	pub fn errors(&self) -> &Vec<InvalidDataPath> {
		&self.1
	}
}
