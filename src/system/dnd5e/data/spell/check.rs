use crate::kdl_ext::NodeContext;
use crate::{
	system::dnd5e::data::{action::AttackKind, Ability},
	utility::NotInList,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum Check {
	AttackRoll(AttackKind),
	SavingThrow(Ability, Option<u8>),
}

impl FromKdl<NodeContext> for Check {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"AttackRoll" => Ok(Self::AttackRoll(AttackKind::from_str(node.next_str_req()?)?)),
			"SavingThrow" => {
				let ability = node.next_str_req_t::<Ability>()?;
				let dc = node.get_i64_opt("dc")?.map(|v| v as u8);
				Ok(Self::SavingThrow(ability, dc))
			}
			name => Err(NotInList(name.into(), vec!["AttackRoll", "SavingThrow"]).into()),
		}
	}
}

impl AsKdl for Check {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::AttackRoll(kind) => NodeBuilder::default().with_entry("AttackRoll").with_entry(match kind {
				AttackKind::Melee => "Melee",
				AttackKind::Ranged => "Ranged",
			}),
			Self::SavingThrow(ability, dc) => {
				let mut node = NodeBuilder::default().with_entry("SavingThrow");
				node.push_entry({
					let mut entry = kdl::KdlEntry::new(ability.long_name());
					entry.set_ty("Ability");
					entry
				});
				if let Some(dc) = dc {
					node.push_entry(("dc", *dc as i64));
				}
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "check";

		#[test]
		fn attack_melee() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" \"Melee\"";
			let data = Check::AttackRoll(AttackKind::Melee);
			assert_eq_fromkdl!(Check, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn attack_ranged() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" \"Ranged\"";
			let data = Check::AttackRoll(AttackKind::Ranged);
			assert_eq_fromkdl!(Check, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn save() -> anyhow::Result<()> {
			let doc = "check \"SavingThrow\" (Ability)\"Dexterity\"";
			let data = Check::SavingThrow(Ability::Dexterity, None);
			assert_eq_fromkdl!(Check, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn save_fixed_dc() -> anyhow::Result<()> {
			let doc = "check \"SavingThrow\" (Ability)\"Dexterity\" dc=15";
			let data = Check::SavingThrow(Ability::Dexterity, Some(15));
			assert_eq_fromkdl!(Check, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
