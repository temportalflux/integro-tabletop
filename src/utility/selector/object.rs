use super::id::IdPath;
use crate::{database::app::Criteria, utility::Value};
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Debug)]
pub struct Object<Context: 'static> {
	pub id: IdPath,
	pub amount: Value<Context, i32>,
	pub object_category: String,
	pub criteria: Option<Criteria>,
}

impl<Context> Object<Context> {
	fn id_path(&self) -> &IdPath {
		&self.id
	}

	pub fn set_data_path(&self, parent: &Path) {
		self.id.set_path(parent);
	}

	pub fn get_data_path(&self) -> Option<PathBuf> {
		self.id.as_path()
	}

	pub fn set_criteria(&mut self, criteria: Criteria) {
		self.criteria = Some(criteria);
	}

	pub fn criteria(&self) -> Option<&Criteria> {
		self.criteria.as_ref()
	}
}

impl<Context> Object<Context>
where
	Context: 'static + Send + Sync,
{
	pub fn as_data(
		&self,
		name: impl Into<String>,
		context: &Context,
	) -> Result<super::DataOption, super::InvalidDataPath> {
		let Some(data_path) = self.id.as_path() else { return Err(super::InvalidDataPath); };
		Ok(super::DataOption {
			data_path,
			name: name.into(),
			kind: super::Kind::Object {
				amount: self.amount.evaluate(context) as usize,
				object_category: self.object_category.clone(),
				criteria: self.criteria.clone(),
			},
		})
	}
}
