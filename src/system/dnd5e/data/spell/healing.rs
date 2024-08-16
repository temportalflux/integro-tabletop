use crate::{
	kdl_ext::NodeContext,
	system::dnd5e::data::roll::{Roll, RollSet},
	utility::NotInList,
};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct Healing {
	rolls: RollSet,
	include_ability_modifier: bool,
	upcast: RollSet,
	pub hide_bonuses_in_overview: bool,
}

impl Healing {
	pub fn rolls(&self) -> &RollSet {
		&self.rolls
	}

	pub fn uses_ability_modifier(&self) -> bool {
		self.include_ability_modifier
	}

	pub fn upcast(&self) -> &RollSet {
		&self.upcast
	}

	pub fn evaluate(&self, modifier: i32, upcast_amount: i32) -> RollSet {
		let mut rolls = self.rolls;
		rolls += self.upcast * upcast_amount;
		if self.include_ability_modifier {
			rolls.push(Roll::from(modifier));
		}
		rolls
	}
}

impl FromKdl<NodeContext> for Healing {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut rolls = RollSet::default();
		let mut include_ability_modifier = false;
		let mut upcast = RollSet::default();
		for mut node in node.query_all("scope() > amount")? {
			let entry = node.next_req()?;
			match entry.type_opt() {
				None => {
					if node.get_bool_opt("ability")? == Some(true) {
						include_ability_modifier = true;
					}
					rolls.extend(RollSet::from_str(entry.as_str_req()?)?);
				}
				Some("Upcast") => {
					upcast.extend(RollSet::from_str(entry.as_str_req()?)?);
				}
				Some(invalid_type) => Err(NotInList(invalid_type.to_owned(), vec!["Upcast"]))?,
			}
		}
		
		let hide_bonuses_in_overview = node.get_bool_opt("hide_bonuses_in_overview")?.unwrap_or(false);

		Ok(Self { rolls, include_ability_modifier, upcast, hide_bonuses_in_overview })
	}
}

impl AsKdl for Healing {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.child(("amount", {
			let mut node = self.rolls.as_kdl();
			if self.include_ability_modifier {
				node.entry(("ability", self.include_ability_modifier));
			}
			node
		}));
		node.child(("amount", &self.upcast));
		if self.hide_bonuses_in_overview {
			node.entry(("hide_bonuses_in_overview", true));
		}
		node
	}
}
