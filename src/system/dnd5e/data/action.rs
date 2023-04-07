use super::{description, IndirectCondition};
use crate::kdl_ext::{DocumentExt, FromKDL, NodeExt};
use std::{
	borrow::Cow,
	path::{Path, PathBuf},
};
use uuid::Uuid;

mod activation;
pub use activation::*;
mod attack;
pub use attack::*;
mod limited_uses;
pub use limited_uses::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Action {
	pub name: String,
	pub description: description::Info,
	pub activation_kind: ActivationKind,
	pub attack: Option<Attack>,
	/// Dictates how many times this action can be used until it is reset.
	pub limited_uses: Option<LimitedUses>,
	/// Conditions applied when the action is used.
	pub conditions_to_apply: Vec<IndirectCondition>,
	// generated
	pub source: Option<ActionSource>,
}

impl Action {
	pub fn set_data_path(&self, parent: &std::path::Path) {
		if let Some(uses) = &self.limited_uses {
			uses.set_data_path(parent);
		}
	}
}

impl FromKDL for Action {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = description::Info::from_kdl_all(node, ctx)?;
		let activation_kind = ActivationKind::from_kdl(
			node.query_req("scope() > activation")?,
			&mut ctx.next_node(),
		)?;

		let attack = match node.query("scope() > attack")? {
			None => None,
			Some(node) => Some(Attack::from_kdl(node, &mut ctx.next_node())?),
		};
		let limited_uses = match node.query("scope() > limited_uses")? {
			None => None,
			Some(node) => Some(LimitedUses::from_kdl(node, &mut ctx.next_node())?),
		};

		let mut conditions_to_apply = Vec::new();
		for node in node.query_all("scope() > condition")? {
			conditions_to_apply.push(IndirectCondition::from_kdl(node, &mut ctx.next_node())?);
		}

		Ok(Self {
			name,
			description,
			activation_kind,
			attack,
			limited_uses,
			conditions_to_apply,
			source: None,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActionSource {
	Item(Uuid),
	Feature(PathBuf),
}
impl ActionSource {
	pub fn as_path<'this>(&'this self, inventory: &super::item::Inventory) -> Cow<'this, Path> {
		match self {
			Self::Feature(path) => Cow::Borrowed(path.as_path()),
			Self::Item(id) => {
				let base = PathBuf::new().join("Equipment");
				let owned = match inventory.get_item(id) {
					Some(item) => base.join(&item.name),
					None => base.join("Unknown Item"),
				};
				Cow::Owned(owned)
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{
				core::SourceId,
				dnd5e::data::{
					roll::{Die, Roll},
					scaling, Ability, Condition, DamageRoll, DamageType, Rest,
				},
			},
			utility,
		};

		fn from_doc(doc: &str) -> anyhow::Result<Action> {
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > action")?
				.expect("missing action node");
			Action::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn basic() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info::default(),
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: Vec::new(),
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn description_simple() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				description \"This is a basic description\"
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info {
					short: None,
					long: vec![description::Section {
						title: None,
						content: "This is a basic description".into(),
					}],
				},
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: Vec::new(),
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn description_multiple() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				description \"Overview with some details\" {
					short \"A brief desc\"
					section (Title)\"Subtitle A\" \"first subsection\"
					section \"another subsection\"
				}
				description (Title)\"Subtitle B\" \"some more details\"
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info {
					short: Some("A brief desc".into()),
					long: vec![
						description::Section {
							title: None,
							content: "Overview with some details".into(),
						},
						description::Section {
							title: Some("Subtitle A".into()),
							content: "first subsection".into(),
						},
						description::Section {
							title: None,
							content: "another subsection".into(),
						},
						description::Section {
							title: Some("Subtitle B".into()),
							content: "some more details".into(),
						},
					],
				},
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: Vec::new(),
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn attack() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				attack {
					kind \"Melee\"
					check \"AttackRoll\" (Ability)\"Dexterity\" proficient=true
					damage base=1 {
						roll (Roll)\"2d6\"
						damage_type (DamageType)\"Fire\"
					}
				}
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info::default(),
				activation_kind: ActivationKind::Action,
				attack: Some(Attack {
					kind: AttackKindValue::Melee { reach: 5 },
					check: AttackCheckKind::AttackRoll {
						ability: Ability::Dexterity,
						proficient: utility::Value::Fixed(true),
					},
					area_of_effect: None,
					damage: Some(DamageRoll {
						roll: Some(Roll {
							amount: 2,
							die: Die::D6,
						}),
						base_bonus: 1,
						damage_type: DamageType::Fire,
						additional_bonuses: Vec::new(),
					}),
				}),
				limited_uses: None,
				conditions_to_apply: Vec::new(),
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn limited_uses_fixed() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				limited_uses {
					max_uses 1
					reset_on \"Long\"
				}
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info::default(),
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: Some(LimitedUses {
					max_uses: scaling::Value::Fixed(1),
					reset_on: Some(Rest::Long),
					..Default::default()
				}),
				conditions_to_apply: Vec::new(),
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn limited_uses_scaling() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				limited_uses {
					max_uses (Scaled)\"Level\" {
						level 2 1
						level 5 2
						level 10 4
						level 14 5
						level 20
					}
					reset_on \"Long\"
				}
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info::default(),
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: Some(LimitedUses {
					max_uses: scaling::Value::Scaled(scaling::Basis::Level {
						class_name: None,
						level_map: [
							(2, Some(1)),
							(5, Some(2)),
							(10, Some(4)),
							(14, Some(5)),
							(20, None),
						]
						.into(),
					}),
					reset_on: Some(Rest::Long),
					..Default::default()
				}),
				conditions_to_apply: Vec::new(),
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn condition_by_id() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				condition \"condition/invisible.kdl\"
				condition \"condition/unconscious.kdl\"
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info::default(),
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: vec![
					IndirectCondition::Id(SourceId {
						path: "condition/invisible.kdl".into(),
						..Default::default()
					}),
					IndirectCondition::Id(SourceId {
						path: "condition/unconscious.kdl".into(),
						..Default::default()
					}),
				],
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn condition_custom() -> anyhow::Result<()> {
			let doc = "action name=\"Action Name\" {
				activation \"Action\"
				condition \"Custom\" name=\"Slippery\"
			}";
			let expected = Action {
				name: "Action Name".into(),
				description: description::Info::default(),
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: vec![IndirectCondition::Custom(Condition {
					name: "Slippery".into(),
					..Default::default()
				})],
				source: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
