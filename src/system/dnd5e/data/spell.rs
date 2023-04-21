use super::{description, AreaOfEffect};
use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeContext, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{DnD5e, SystemComponent},
	},
};

mod casting_time;
pub use casting_time::*;
mod check;
pub use check::*;
mod component;
pub use component::*;
mod damage;
pub use damage::*;
mod duration;
pub use duration::*;
mod range;
pub use range::*;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Spell {
	pub name: String,
	pub description: description::Info,
	pub rank: u8,
	pub components: Components,
	pub casting_time: CastingTime,
	pub range: Range,
	pub check: Option<Check>,
	pub damage: Option<Damage>,
	pub area_of_effect: Option<AreaOfEffect>,
	pub duration: Duration,
	pub tags: Vec<String>,
}

crate::impl_kdl_node!(Spell, "spell");

impl SystemComponent for Spell {
	type System = DnD5e;

	fn add_component(self, _source_id: SourceId, _system: &mut Self::System) {}
}

impl FromKDL for Spell {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = description::Info::from_kdl_all(node, ctx)?;
		let rank = node.query_i64_req("scope() > rank", 0)? as u8;

		let components = Components::from_kdl_all(node, ctx)?;
		let casting_time = node.query_req("scope() > casting-time")?;
		let casting_time = CastingTime::from_kdl(casting_time, &mut ctx.next_node())?;
		let range = node.query_req("scope() > range")?;
		let range = Range::from_kdl(range, &mut ctx.next_node())?;
		let area_of_effect = match node.query_opt("scope() > area_of_effect")? {
			None => None,
			Some(node) => Some(AreaOfEffect::from_kdl(node, &mut ctx.next_node())?),
		};
		let duration = node.query_req("scope() > duration")?;
		let duration = Duration::from_kdl(duration, &mut ctx.next_node())?;

		let check = match node.query_opt("scope() > check")? {
			None => None,
			Some(node) => Some(Check::from_kdl(node, &mut ctx.next_node())?),
		};
		let damage = match node.query_opt("scope() > damage")? {
			None => None,
			Some(node) => Some(Damage::from_kdl(node, &mut ctx.next_node())?),
		};

		let tags = node.query_str_all("scope() > tag", 0)?;
		let tags = tags.into_iter().map(str::to_owned).collect();

		Ok(Self {
			name,
			description,
			rank,
			components,
			casting_time,
			range,
			check,
			damage,
			area_of_effect,
			duration,
			tags,
		})
	}
}
