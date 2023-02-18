use super::inventory::{ArmorType, WeaponType};
use crate::system::dnd5e::{character::DerivedBuilder, mutator::Selector};
use std::{
	collections::{BTreeMap, BTreeSet},
	path::PathBuf,
};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct OtherProficiencies {
	pub languages: AttributedValueMap<String>,
	pub armor: AttributedValueMap<ArmorType>,
	pub weapons: AttributedValueMap<WeaponProficiency>,
	pub tools: AttributedValueMap<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WeaponProficiency {
	Kind(WeaponType),
	Classification(String),
}
impl ToString for WeaponProficiency {
	fn to_string(&self) -> String {
		match self {
			Self::Kind(WeaponType::Simple) => "Simple Weapons".into(),
			Self::Kind(WeaponType::Martial) => "Martial Weapons".into(),
			Self::Classification(name) => name.clone(),
		}
	}
}

#[derive(Clone)]
pub enum AddProficiency {
	Language(Selector<String>),
	Armor(ArmorType),
	Weapon(WeaponProficiency),
	Tool(String),
}

impl super::mutator::Mutator for AddProficiency {
	fn scope_id(&self) -> Option<&str> {
		match self {
			Self::Language(selector) => selector.id(),
			_ => None,
		}
	}
	
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		let scope = stats.scope();
		match &self {
			Self::Language(value) => {
				let value = match value {
					Selector::Specific(value) => Some(value.clone()),
					_ => stats.get_selection().map(str::to_owned),
				};
				if let Some(value) = value {
					stats.other_proficiencies.languages.insert(value, scope);
				}
			}
			Self::Armor(value) => {
				stats.other_proficiencies.armor.insert(value.clone(), scope);
			}
			Self::Weapon(value) => {
				stats
					.other_proficiencies
					.weapons
					.insert(value.clone(), scope);
			}
			Self::Tool(value) => {
				stats.other_proficiencies.tools.insert(value.clone(), scope);
			}
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
