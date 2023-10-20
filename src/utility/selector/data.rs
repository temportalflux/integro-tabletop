use crate::database::Criteria;
use enumset::EnumSetType;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone, PartialEq, Debug)]
pub struct DataOption {
	pub data_path: PathBuf,
	pub name: String,
	pub kind: Kind,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
	StringEntry {
		amount: usize,
		options: BTreeSet<String>,
		blocked_options: BTreeSet<String>,
		cannot_match: Vec<PathBuf>,
	},
	Object {
		amount: usize,
		object_category: String,
		criteria: Option<Criteria>,
	},
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct DataList(Vec<DataOption>, Vec<super::InvalidDataPath>);

impl DataList {
	pub fn with_value<Context, T>(
		mut self,
		name: impl Into<String>,
		selector: &super::Value<Context, T>,
		context: Option<&Context>,
	) -> Self
	where
		Context: 'static + Send + Sync,
		T: 'static + Clone + Send + Sync + std::fmt::Debug + PartialEq + ToString,
	{
		let Some(context) = context else {
			return self;
		};
		match selector.as_data(name, context) {
			Ok(None) => {}
			Ok(Some(data)) => self.0.push(data),
			Err(err) => self.1.push(err),
		}
		self
	}

	pub fn with_enum<Context, T>(
		mut self,
		name: impl Into<String>,
		selector: &super::Value<Context, T>,
		context: Option<&Context>,
	) -> Self
	where
		Context: 'static + Send + Sync + Clone,
		T: 'static
			+ Clone
			+ Send
			+ Sync
			+ std::fmt::Debug
			+ PartialEq
			+ ToString
			+ std::str::FromStr
			+ EnumSetType
			+ Ord,
	{
		let Some(context) = context else {
			return self;
		};
		match selector.with_default_options().as_data(name, context) {
			Ok(None) => {}
			Ok(Some(data)) => self.0.push(data),
			Err(err) => self.1.push(err),
		}
		self
	}

	pub fn with_object<Context>(
		mut self,
		name: impl Into<String>,
		selector: &super::Object<Context>,
		context: Option<&Context>,
	) -> Self
	where
		Context: 'static + Send + Sync,
	{
		let Some(context) = context else {
			return self;
		};
		match selector.as_data(name, context) {
			Ok(data) => self.0.push(data),
			Err(err) => self.1.push(err),
		}
		self
	}

	pub fn as_vec(&self) -> &Vec<DataOption> {
		&self.0
	}

	pub fn errors(&self) -> &Vec<super::InvalidDataPath> {
		&self.1
	}
}
