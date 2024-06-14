use crate::GeneralError;
use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Enum, EnumSetType, PartialOrd, Ord, Hash)]
pub enum Modifier {
	Advantage,
	Disadvantage,
}
impl Modifier {
	pub fn display_name(&self) -> &'static str {
		match self {
			Modifier::Advantage => "Advantage",
			Modifier::Disadvantage => "Disadvantage",
		}
	}
}
impl ToString for Modifier {
	fn to_string(&self) -> String {
		self.display_name().to_owned()
	}
}
impl FromStr for Modifier {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Advantage" => Ok(Self::Advantage),
			"Disadvantage" => Ok(Self::Disadvantage),
			_ => Err(GeneralError(format!(
				"Invalid roll modifier value {s:?}, expected Advantage or Disadvantage."
			))),
		}
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ModifierList(EnumMap<Modifier, Vec<(Option<String>, PathBuf)>>);

impl std::ops::Index<Modifier> for ModifierList {
	type Output = Vec<(Option<String>, PathBuf)>;
	fn index(&self, index: Modifier) -> &Self::Output {
		&self.0[index]
	}
}

impl ModifierList {
	pub fn push(&mut self, modifier: Modifier, context: Option<String>, source: PathBuf) {
		self.0[modifier].push((context, source));
	}

	pub fn iter(&self) -> impl Iterator<Item = (Modifier, &Vec<(Option<String>, PathBuf)>)> {
		self.0.iter()
	}

	pub fn iter_all(&self) -> impl Iterator<Item = (Modifier, &Option<String>, &PathBuf)> {
		let iter = self.0.iter();
		let iter = iter.map(|(modifier, items)| items.iter().map(move |(context, source)| (modifier, context, source)));
		iter.flatten()
	}
}
