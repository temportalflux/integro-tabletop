use crate::{
	kdl_ext::{NodeExt, ValueExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{character::Character, DamageType},
			FromKDL, Value,
		},
	},
	utility::Mutator,
	GeneralError,
};
use enum_map::Enum;
use std::str::FromStr;

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
crate::impl_trait_eq!(AddDefense);
crate::impl_kdl_node!(AddDefense, "add_defense");
impl Mutator for AddDefense {
	type Target = Character;

	// TODO: mutator description add_defense

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		stats.defenses_mut().push(
			self.defense,
			self.damage_type.clone(),
			self.context.clone(),
			parent.to_owned(),
		);
	}
}

impl FromKDL for AddDefense {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let defense = Defense::from_str(node.get_str_req(value_idx.next())?)?;
		let damage_type = match node.entry(value_idx.next()) {
			Some(entry) => Some(Value::from_kdl(node, entry, value_idx, node_reg, |kdl| {
				Ok(DamageType::from_str(kdl.as_str_req()?)?)
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
	use crate::system::{
		core::NodeRegistry,
		dnd5e::{
			data::{
				character::{Character, DefenseEntry, Persistent},
				DamageType, Feature,
			},
			BoxedMutator, Value,
		},
	};

	fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
		let mut node_reg = NodeRegistry::default();
		node_reg.register_mutator::<AddDefense>();
		node_reg.parse_kdl_mutator(doc)
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
		fn damage_type() -> anyhow::Result<()> {
			let doc = "mutator \"add_defense\" (Defense)\"Resistance\" (DamageType)\"Cold\"";
			let expected = AddDefense {
				defense: Defense::Resistance,
				damage_type: Some(Value::Fixed(DamageType::Cold)),
				context: None,
			};
			assert_eq!(from_doc(doc)?, expected.into());
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
