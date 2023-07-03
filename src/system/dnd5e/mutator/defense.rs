use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{character::Character, description, DamageType},
	utility::{InvalidEnumStr, Mutator},
};
use enum_map::Enum;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(EnumSetType, Enum, Debug)]
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
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Resistance" => Ok(Self::Resistance),
			"Immunity" => Ok(Self::Immunity),
			"Vulnerability" => Ok(Self::Vulnerability),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddDefense {
	pub defense: Defense,
	pub damage_type: Option<DamageType>,
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

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			content: format!(
				"You are {} to {} damage{}.",
				match self.defense {
					Defense::Resistance => "resistant",
					Defense::Immunity => "immune",
					Defense::Vulnerability => "vulnerable",
				},
				match &self.damage_type {
					None => "all",
					Some(damage_type) => damage_type.display_name(),
				},
				self.context
					.as_ref()
					.map(|ctx| format!(" from {ctx}"))
					.unwrap_or_default(),
			)
			.into(),
			..Default::default()
		}
	}

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
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let defense = Defense::from_str(node.next_str_req()?)?;
		let damage_type = match node.next_str_opt()? {
			Some(str) => Some(DamageType::from_str(str)?),
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

impl AsKdl for AddDefense {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(self.defense.to_string());
		if let Some(damage_type) = &self.damage_type {
			node.push_entry_typed(damage_type.to_string(), "DamageType");
		}
		if let Some(context) = &self.context {
			node.push_entry(("context", context.clone()));
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::{
		character::{Character, DefenseEntry, Persistent},
		Bundle, DamageType,
	};

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(AddDefense);

		#[test]
		fn no_args() -> anyhow::Result<()> {
			let doc = "mutator \"add_defense\" \
				\"Resistance\" context=\"nonmagical attacks\"";
			let data = AddDefense {
				defense: Defense::Resistance,
				damage_type: None,
				context: Some("nonmagical attacks".into()),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn damage_type() -> anyhow::Result<()> {
			let doc = "mutator \"add_defense\" \
				\"Resistance\" (DamageType)\"Cold\"";
			let data = AddDefense {
				defense: Defense::Resistance,
				damage_type: Some(DamageType::Cold),
				context: None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	#[test]
	fn resistant() {
		let character = Character::from(Persistent {
			bundles: vec![Bundle {
				name: "AddDefense".into(),
				mutators: vec![AddDefense {
					defense: Defense::Resistance,
					damage_type: Some(DamageType::Fire),
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
				damage_type: Some(DamageType::Fire),
				context: None,
				source: "AddDefense".into(),
			}]
		);
	}

	#[test]
	fn immune() {
		let character = Character::from(Persistent {
			bundles: vec![Bundle {
				name: "AddDefense".into(),
				mutators: vec![AddDefense {
					defense: Defense::Immunity,
					damage_type: Some(DamageType::Cold),
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
				damage_type: Some(DamageType::Cold),
				context: None,
				source: "AddDefense".into(),
			}]
		);
	}

	#[test]
	fn vulnerable() {
		let character = Character::from(Persistent {
			bundles: vec![Bundle {
				name: "AddDefense".into(),
				mutators: vec![AddDefense {
					defense: Defense::Vulnerability,
					damage_type: Some(DamageType::Psychic),
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
				damage_type: Some(DamageType::Psychic),
				context: None,
				source: "AddDefense".into(),
			}]
		);
	}
}
