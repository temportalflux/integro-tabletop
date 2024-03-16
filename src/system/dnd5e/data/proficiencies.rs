use crate::system::{
	dnd5e::data::item::{armor, weapon},
	mutator::ReferencePath,
};
use std::{
	collections::{BTreeMap, BTreeSet},
	path::PathBuf,
	str::FromStr,
};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct OtherProficiencies {
	pub languages: AttributedValueMap<String>,
	pub armor: AttributedValueMap<(ArmorExtended, Option<String>)>,
	pub weapons: AttributedValueMap<WeaponProficiency>,
	pub tools: AttributedValueMap<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArmorExtended {
	Kind(armor::Kind),
	Shield,
}
impl From<armor::Kind> for ArmorExtended {
	fn from(value: armor::Kind) -> Self {
		Self::Kind(value)
	}
}
impl ToString for ArmorExtended {
	fn to_string(&self) -> String {
		match self {
			Self::Kind(kind) => kind.to_string(),
			Self::Shield => "Shield".into(),
		}
	}
}
impl FromStr for ArmorExtended {
	type Err = <armor::Kind as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Shield" => Ok(Self::Shield),
			_ => Ok(Self::Kind(armor::Kind::from_str(s)?)),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WeaponProficiency {
	Kind(weapon::Kind),
	Classification(String),
}
impl ToString for WeaponProficiency {
	fn to_string(&self) -> String {
		match self {
			Self::Kind(kind) => kind.to_string(),
			Self::Classification(name) => name.clone(),
		}
	}
}
impl WeaponProficiency {
	pub fn display_name(&self) -> String {
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
	pub fn insert(&mut self, value: T, source: &ReferencePath)
	where
		T: Ord,
	{
		match self.0.get_mut(&value) {
			Some(sources) => {
				sources.insert(source.display.clone());
			}
			None => {
				self.0.insert(value, BTreeSet::from([source.display.clone()]));
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
