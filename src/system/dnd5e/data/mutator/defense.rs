use std::str::FromStr;

use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Mutator,
	GeneralError,
};
use enum_map::Enum;

#[derive(Clone, Copy, PartialEq, Enum, Debug)]
pub enum Defense {
	Resistance,
	Immunity,
	Vulnerability,
}
impl ToString for Defense {
	fn to_string(&self) -> String {
		match self {
			Self::Resistance => "Resistance",
			Self::Immunity => "Immunity",
			Self::Vulnerability => "Vulnerability",
		}
		.into()
	}
}
impl FromStr for Defense {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Resistance" => Ok(Self::Resistance),
			"Immunity" => Ok(Self::Immunity),
			"Vulnerability" => Ok(Self::Vulnerability),
			_ => Err(GeneralError(format!(
				"Invalid Defense {s:?}. Expected: Resistance, Immunity, or Vulnerability"
			))),
		}
	}
}

/*TODO: its a defense to a damage type and/or some context (e.g. Cold Damage, Ranged Attacks, Fire Damage from Ranged Attacks)*/
#[derive(Clone, Debug)]
pub struct AddDefense(pub Defense, pub String);
impl KDLNode for AddDefense {
	fn id() -> &'static str {
		"add_defense"
	}
}
impl Mutator for AddDefense {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.defenses_mut().push(self.0, self.1.clone(), source);
	}
}

impl FromKDL<DnD5e> for AddDefense {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let defense = Defense::from_str(node.get_str(value_idx.next())?)?;
		let context = node.get_str(value_idx.next())?.to_owned();
		Ok(Self(defense, context))
	}
}

#[cfg(test)]
mod test {
	use super::{AddDefense, Defense};
	use crate::system::dnd5e::data::{
		character::{Character, Persistent},
		Feature,
	};

	// TODO: Test AddDefense FromKDL

	#[test]
	fn resistant() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense(Defense::Resistance, "Fire".into()).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Resistance],
			[("Fire".into(), ["AddDefense".into()].into())].into()
		);
	}

	#[test]
	fn immune() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense(Defense::Immunity, "Cold".into()).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Immunity],
			[("Cold".into(), ["AddDefense".into()].into())].into()
		);
	}

	#[test]
	fn vulnerable() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense(Defense::Vulnerability, "Psychic".into()).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Vulnerability],
			[("Psychic".into(), ["AddDefense".into()].into())].into()
		);
	}
}
