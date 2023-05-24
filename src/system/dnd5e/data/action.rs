use super::IndirectCondition;
use crate::kdl_ext::{DocumentExt, FromKDL, NodeExt};
use std::str::FromStr;

mod activation;
pub use activation::*;
mod attack;
pub use attack::*;
mod limited_uses;
pub use limited_uses::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Action {
	pub activation_kind: ActivationKind,
	pub attack: Option<Attack>,
	/// Dictates how many times this action can be used until it is reset.
	pub limited_uses: Option<LimitedUses>,
	/// Conditions applied when the action is used.
	pub conditions_to_apply: Vec<IndirectCondition>,
}

impl FromKDL for Action {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let activation_kind = match (
			node.get_str_opt(ctx.consume_idx())?,
			node.query_opt("scope() > activation")?,
		) {
			(Some(str), None) => ActivationKind::from_str(str)?,
			(None, Some(node)) => ActivationKind::from_kdl(node, &mut ctx.next_node())?,
			_ => return Err(MissingActivation(node.to_string()).into()),
		};

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
			activation_kind,
			attack,
			limited_uses,
			conditions_to_apply,
		})
	}
}

#[derive(thiserror::Error, Debug)]
#[error("Action node is missing activation property: {0:?}")]
pub struct MissingActivation(String);

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{
				core::{SourceId, NodeRegistry},
				dnd5e::{
					data::{
						roll::{Die, Roll},
						Ability, Condition, DamageRoll, DamageType, Rest,
					},
					evaluator::GetLevel,
					Value,
				},
			},
			utility,
		};

		fn from_doc(doc: &str) -> anyhow::Result<Action> {
			let mut ctx = NodeContext::registry(NodeRegistry::default_with_eval::<GetLevel>());
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > action")?
				.expect("missing action node");
			Action::from_kdl(node, &mut ctx)
		}

		#[test]
		fn basic() -> anyhow::Result<()> {
			let doc = "action {
				activation \"Action\"
			}";
			let expected = Action {
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: Vec::new(),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn attack() -> anyhow::Result<()> {
			let doc = "action {
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
				activation_kind: ActivationKind::Action,
				attack: Some(Attack {
					kind: AttackKindValue::Melee { reach: 5 },
					check: AttackCheckKind::AttackRoll {
						ability: Ability::Dexterity,
						proficient: utility::Value::Fixed(true),
					},
					area_of_effect: None,
					damage: Some(DamageRoll {
						roll: Some(Roll::from((2, Die::D6))),
						base_bonus: 1,
						damage_type: DamageType::Fire,
						additional_bonuses: Vec::new(),
					}),
					weapon_kind: None,
				}),
				limited_uses: None,
				conditions_to_apply: Vec::new(),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn limited_uses_fixed() -> anyhow::Result<()> {
			let doc = "action {
				activation \"Action\"
				limited_uses {
					max_uses 1
					reset_on \"Long\"
				}
			}";
			let expected = Action {
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: Some(LimitedUses {
					max_uses: Value::Fixed(1),
					reset_on: Some(Rest::Long),
					..Default::default()
				}),
				conditions_to_apply: Vec::new(),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn limited_uses_scaling() -> anyhow::Result<()> {
			let doc = "action {
				activation \"Action\"
				limited_uses {
					max_uses (Evaluator)\"get_level\" {
						level 2 1
						level 5 2
						level 10 4
						level 14 5
						level 20 -1
					}
					reset_on \"Long\"
				}
			}";
			let expected = Action {
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: Some(LimitedUses {
					max_uses: Value::Evaluated(
						GetLevel {
							class_name: None,
							order_map: [(2, 1), (5, 2), (10, 4), (14, 5), (20, -1)].into(),
						}
						.into(),
					),
					reset_on: Some(Rest::Long),
					..Default::default()
				}),
				conditions_to_apply: Vec::new(),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn condition_by_id() -> anyhow::Result<()> {
			let doc = "action {
				activation \"Action\"
				condition \"condition/invisible.kdl\"
				condition \"condition/unconscious.kdl\"
			}";
			let expected = Action {
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
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn condition_custom() -> anyhow::Result<()> {
			let doc = "action {
				activation \"Action\"
				condition \"Custom\" name=\"Slippery\"
			}";
			let expected = Action {
				activation_kind: ActivationKind::Action,
				attack: None,
				limited_uses: None,
				conditions_to_apply: vec![IndirectCondition::Custom(Condition {
					name: "Slippery".into(),
					..Default::default()
				})],
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
