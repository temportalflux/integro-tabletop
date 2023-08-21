use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	system::dnd5e::data::{character::Character, description, DamageType},
	utility::{list_as_english, selector, InvalidEnumStr, Mutator},
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
	pub damage_type: Option<selector::Value<Character, DamageType>>,
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

	fn set_data_path(&self, parent: &std::path::Path) {
		if let Some(selector) = &self.damage_type {
			selector.set_data_path(parent);
		}
	}

	fn description(&self, state: Option<&Character>) -> description::Section {
		let body = format!(
			"You are {} to {} damage{}.",
			match self.defense {
				Defense::Resistance => "resistant",
				Defense::Immunity => "immune",
				Defense::Vulnerability => "vulnerable",
			},
			match &self.damage_type {
				None => "all".to_owned(),
				Some(selector::Value::Specific(damage_type)) =>
					damage_type.display_name().to_owned(),
				Some(selector::Value::Options { options, .. }) if options.is_empty() =>
					"any single type of".to_owned(),
				Some(selector::Value::Options { options, .. }) => {
					let options = options.iter().map(DamageType::to_string).collect();
					list_as_english(options, "or").unwrap_or_default()
				}
			},
			self.context
				.as_ref()
				.map(|ctx| format!(" from {ctx}"))
				.unwrap_or_default(),
		);
		let mut selectors = selector::DataList::default();
		if let Some(damage_type) = &self.damage_type {
			selectors = selectors.with_enum("Damage Type", damage_type, state);
		}
		description::Section {
			content: body.into(),
			children: vec![selectors.into()],
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		let damage_type = match &self.damage_type {
			None => None,
			Some(selector) => stats.resolve_selector(selector),
		};
		stats.defenses_mut().push(
			self.defense,
			damage_type,
			self.context.clone(),
			parent.to_owned(),
		);
	}
}

impl FromKDL for AddDefense {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let defense = node.next_str_req_t::<Defense>()?;
		let damage_type = match node.peak_opt().is_some() {
			true => Some(selector::Value::from_kdl(node)?),
			false => None,
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
			node.append_typed("DamageType", damage_type.as_kdl());
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
		use crate::{
			kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils, utility::Value,
		};

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
		fn specific() -> anyhow::Result<()> {
			let doc = "mutator \"add_defense\" \
				\"Resistance\" (DamageType)\"Specific\" \"Cold\"";
			let data = AddDefense {
				defense: Defense::Resistance,
				damage_type: Some(selector::Value::Specific(DamageType::Cold)),
				context: None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn any() -> anyhow::Result<()> {
			let doc = "mutator \"add_defense\" \"Resistance\" (DamageType)\"Any\"";
			let data = AddDefense {
				defense: Defense::Resistance,
				damage_type: Some(selector::Value::Options {
					id: Default::default(),
					options: Default::default(),
					amount: Value::Fixed(1),
					is_applicable: None,
				}),
				context: None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn any_of() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_defense\" \"Resistance\" (DamageType)\"AnyOf\" {
				|    option \"Fire\"
				|    option \"Force\"
				|}
			";
			let data = AddDefense {
				defense: Defense::Resistance,
				damage_type: Some(selector::Value::Options {
					id: Default::default(),
					options: [DamageType::Fire, DamageType::Force].into(),
					amount: Value::Fixed(1),
					is_applicable: None,
				}),
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
					damage_type: Some(selector::Value::Specific(DamageType::Fire)),
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
					damage_type: Some(selector::Value::Specific(DamageType::Cold)),
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
					damage_type: Some(selector::Value::Specific(DamageType::Psychic)),
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
