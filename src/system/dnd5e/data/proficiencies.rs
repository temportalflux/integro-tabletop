use crate::system::dnd5e::data::item::{armor, weapon};
use std::{
	collections::{BTreeMap, BTreeSet},
	path::PathBuf,
};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct OtherProficiencies {
	pub languages: AttributedValueMap<String>,
	pub armor: AttributedValueMap<armor::Kind>,
	pub weapons: AttributedValueMap<WeaponProficiency>,
	pub tools: AttributedValueMap<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WeaponProficiency {
	Kind(weapon::Kind),
	Classification(String),
}
impl ToString for WeaponProficiency {
	fn to_string(&self) -> String {
		match self {
			Self::Kind(weapon::Kind::Simple) => "Simple Weapons".into(),
			Self::Kind(weapon::Kind::Martial) => "Martial Weapons".into(),
			Self::Classification(name) => name.clone(),
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct AttributedValueMap<T>(BTreeMap<T, BTreeSet<PathBuf>>);
impl<T> Default for AttributedValueMap<T> {
	fn default() -> Self {
		Self(BTreeMap::new())
	}
}
impl<T> From<BTreeMap<T, BTreeSet<PathBuf>>> for AttributedValueMap<T> {
	fn from(value: BTreeMap<T, BTreeSet<PathBuf>>) -> Self {
		Self(value)
	}
}
impl<T> AttributedValueMap<T> {
	pub fn insert(&mut self, value: T, source: PathBuf)
	where
		T: Ord,
	{
		match self.0.get_mut(&value) {
			Some(sources) => {
				sources.insert(source);
			}
			None => {
				self.0.insert(value, BTreeSet::from([source]));
			}
		}
	}
}
impl<T> std::ops::Deref for AttributedValueMap<T> {
	type Target = BTreeMap<T, BTreeSet<PathBuf>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
