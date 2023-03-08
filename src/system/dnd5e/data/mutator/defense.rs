use std::str::FromStr;

use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{
		data::{action::DamageType, character::Character},
		DnD5e, FromKDL, KDLNode, Value,
	},
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

#[derive(Clone, Debug, PartialEq)]
pub struct AddDefense {
	pub defense: Defense,
	pub damage_type: Option<Value<DamageType>>,
	pub context: Option<String>,
}
impl Default for AddDefense {
	fn default() -> Self {
		Self {
			defense: Defense::Resistance,
			damage_type: Default::default(),
			context: Default::default(),
		}
	}
}
impl crate::utility::TraitEq for AddDefense {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}
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
		stats.defenses_mut().push(
			self.defense,
			self.damage_type.clone(),
			self.context.clone(),
			source,
		);
	}
}

impl FromKDL<DnD5e> for AddDefense {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let defense = Defense::from_str(node.get_str(value_idx.next())?)?;
		let damage_type = match node.entry("damage_type") {
			Some(entry) => Some(Value::from_kdl(node, entry, value_idx, system, |kdl| {
				Ok(match kdl.as_string() {
					None => None,
					Some(str) => Some(DamageType::from_str(str)?),
				})
			})?),
			None => None,
		};
		let context = node.get_str_opt("context")?.map(str::to_owned);
		Ok(Self {
			defense,
			damage_type,
			context,
		})
	}
}

#[cfg(test)]
mod test {
	use super::{AddDefense, Defense};
	use crate::system::dnd5e::{
		data::{
			action::DamageType,
			character::{Character, DefenseEntry, Persistent},
			evaluator::ByLevel,
			Feature,
		},
		BoxedMutator, DnD5e, Value,
	};

	fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
		let mut system = DnD5e::default();
		system.register_mutator::<AddDefense>();
		system.register_evaluator::<ByLevel>();
		system.parse_kdl_mutator(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn no_args() -> anyhow::Result<()> {
			let doc = "mutator \"add_defense\" (Defense)\"Resistance\"";
			let expected = AddDefense {
				defense: Defense::Resistance,
				damage_type: None,
				context: None,
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn eval_damage_type() -> anyhow::Result<()> {
			/* TODO
			let doc = "mutator \"add_defense\" (Defense)\"Resistance\" damage_type=(Evaluator)\"map_level\"";
			let expected = AddDefense {
				defense: Defense::Resistance,
				damage_type: Some(Value::Evaluated(
					ByLevel {
						class_name: None,
						map: [].into(),
					}
					.into(),
				)),
				context: None,
			};
			assert_eq!(from_doc(doc)?, expected.into());
			*/
			Ok(())
		}
	}

	#[test]
	fn resistant() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense {
					defense: Defense::Resistance,
					damage_type: Some(Value::Fixed(DamageType::Fire)),
					context: None,
				}
				.into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Resistance],
			vec![DefenseEntry {
				damage_type: Some(Value::Fixed(DamageType::Fire)),
				context: None,
				source: "AddDefense".into(),
			}]
		);
	}

	#[test]
	fn immune() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense {
					defense: Defense::Immunity,
					damage_type: Some(Value::Fixed(DamageType::Cold)),
					context: None,
				}
				.into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Immunity],
			vec![DefenseEntry {
				damage_type: Some(Value::Fixed(DamageType::Cold)),
				context: None,
				source: "AddDefense".into(),
			}]
		);
	}

	#[test]
	fn vulnerable() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense {
					defense: Defense::Vulnerability,
					damage_type: Some(Value::Fixed(DamageType::Psychic)),
					context: None,
				}
				.into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Vulnerability],
			vec![DefenseEntry {
				damage_type: Some(Value::Fixed(DamageType::Psychic)),
				context: None,
				source: "AddDefense".into(),
			}]
		);
	}
}
