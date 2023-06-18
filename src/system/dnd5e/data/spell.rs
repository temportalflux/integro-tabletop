use super::{description, AreaOfEffect};
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeContext, NodeExt},
	system::{core::SourceId, dnd5e::SystemComponent},
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
	pub id: SourceId,
	pub name: String,
	pub description: description::Info,
	pub rank: u8,
	pub school_tag: Option<String>,
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
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name.clone(),
			"tags": self.tags.clone(),
			"rank": self.rank,
			"school": self.school_tag.clone(),
			"components": {
				"verbal": self.components.verbal,
				"somatic": self.components.somatic,
				"material": self.components.materials.len() > 0,
			},
			"casting": {
				"duration": self.casting_time.duration.as_metadata(),
				"ritual": self.casting_time.ritual,
			},
			"duration": self.duration.kind.as_metadata(),
			"concentration": self.duration.concentration,
		})
	}
}

impl FromKDL for Spell {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		// TODO: all system components need to check if the node has a source field
		let id = ctx.parse_source_req(node)?;

		let name = node.get_str_req("name")?.to_owned();
		let description = match node.query_opt("scope() > description")? {
			None => description::Info::default(),
			Some(node) => description::Info::from_kdl(node, &mut ctx.next_node())?,
		};
		let rank = node.query_i64_req("scope() > rank", 0)? as u8;
		let school_tag = node
			.query_str_opt("scope() > school", 0)?
			.map(str::to_owned);

		let components = Components::from_kdl(node, ctx)?;
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

		let mut tags = node.query_str_all("scope() > tag", 0)?;
		tags.sort();
		let tags = tags.into_iter().map(str::to_owned).collect();

		Ok(Self {
			id,
			name,
			description,
			rank,
			school_tag,
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
// TODO AsKdl: from/as tests for Spell
impl AsKdl for Spell {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("name", self.name.clone()));
		node.push_child_opt_t("source", &self.id);

		if let Some(school) = &self.school_tag {
			node.push_child_opt_t("school", school);
		}
		for tag in &self.tags {
			node.push_child_opt_t("tag", tag);
		}
		node.push_child_t("rank", &self.rank);

		node.push_child_t("casting-time", &self.casting_time);
		node.push_child_t("range", &self.range);
		if let Some(area_of_effect) = &self.area_of_effect {
			node.push_child_t("area_of_effect", area_of_effect);
		}

		node += self.components.as_kdl();

		node.push_child_t("duration", &self.duration);
		if let Some(check) = &self.check {
			node.push_child_t("check", check);
		}
		if let Some(damage) = &self.damage {
			node.push_child_t("damage", damage);
		}

		node.push_child_opt_t("description", &self.description);

		node
	}
}
