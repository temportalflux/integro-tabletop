use crate::system::dnd5e::{
	data::{
		character::Character,
		item::{armor, weapon},
	},
	mutator::{Mutator, Selector},
};
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

#[derive(Clone)]
pub enum AddProficiency {
	Language(Selector<String>),
	Armor(armor::Kind),
	Weapon(WeaponProficiency),
	Tool(String),
}

impl Mutator for AddProficiency {
	fn node_id(&self) -> &'static str {
		"add_proficiency"
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		match &self {
			Self::Language(value) => {
				if let Some(value) = stats.resolve_selector(value) {
					stats
						.other_proficiencies_mut()
						.languages
						.insert(value, source);
				}
			}
			Self::Armor(value) => {
				stats
					.other_proficiencies_mut()
					.armor
					.insert(value.clone(), source);
			}
			Self::Weapon(value) => {
				stats
					.other_proficiencies_mut()
					.weapons
					.insert(value.clone(), source);
			}
			Self::Tool(value) => {
				stats
					.other_proficiencies_mut()
					.tools
					.insert(value.clone(), source);
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
