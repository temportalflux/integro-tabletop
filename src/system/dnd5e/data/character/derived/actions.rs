use crate::{
	path_map::PathMap,
	system::dnd5e::data::{action::UseCounterData, Feature},
	utility::NotInList,
};
use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	str::FromStr,
};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Features {
	pub path_map: PathMap<Feature>,
	pub uses: HashMap<PathBuf, UseCounterData>,
	pub action_budget: ActionBudget,
}

impl Features {
	pub fn register_usage(&mut self, usage_data: &UseCounterData, path: impl AsRef<Path>) {
		self.uses.insert(path.as_ref().to_owned(), usage_data.clone());
	}

	pub fn iter_all(&self) -> impl Iterator<Item = (PathBuf, &Feature)> + '_ {
		self.path_map.as_vec().into_iter()
	}

	pub fn get_usage(&self, key: impl AsRef<Path>) -> Option<&UseCounterData> {
		self.uses.get(key.as_ref())
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct ActionBudget(EnumMap<ActionBudgetKind, (u32, Vec<(u32, PathBuf)>)>);

impl Default for ActionBudget {
	fn default() -> Self {
		Self(enum_map::enum_map! {
			ActionBudgetKind::Action => (1, Vec::new()),
			ActionBudgetKind::Attack => (1, Vec::new()),
			ActionBudgetKind::Bonus => (1, Vec::new()),
			ActionBudgetKind::Reaction => (1, Vec::new()),
		})
	}
}

impl ActionBudget {
	pub fn get(&self, kind: ActionBudgetKind) -> (u32, &Vec<(u32, PathBuf)>) {
		let (amt, sources) = &self.0[kind];
		(*amt, sources)
	}

	pub fn push(&mut self, kind: ActionBudgetKind, amount: u32, source: PathBuf) {
		self.0[kind].0 += amount;
		self.0[kind].1.push((amount, source));
	}
}

#[derive(Debug, EnumSetType, Enum)]
pub enum ActionBudgetKind {
	Action,
	Attack,
	Bonus,
	Reaction,
}
impl ToString for ActionBudgetKind {
	fn to_string(&self) -> String {
		match self {
			Self::Action => "Action",
			Self::Attack => "Attack",
			Self::Bonus => "Bonus",
			Self::Reaction => "Reaction",
		}
		.into()
	}
}
impl FromStr for ActionBudgetKind {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Action" => Ok(Self::Action),
			"Attack" => Ok(Self::Attack),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			_ => Err(NotInList(s.into(), vec!["Action", "Bonus", "Reaction", "Attack"]).into()),
		}
	}
}
