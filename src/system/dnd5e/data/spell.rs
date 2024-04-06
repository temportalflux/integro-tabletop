use super::{description, AreaOfEffect};
use crate::kdl_ext::NodeContext;
use crate::system::{Block, SourceId};
use kdlize::OmitIfEmpty;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};

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

kdlize::impl_kdl_node!(Spell, "spell");

impl Block for Spell {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"id": self.id.unversioned().to_string(),
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

impl FromKdl<NodeContext> for Spell {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		// TODO: all system components need to check if the node has a source field
		let id = crate::kdl_ext::query_source_req(node)?;

		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_opt_t::<description::Info>("scope() > description")?
			.unwrap_or_default();
		let rank = node.query_i64_req("scope() > rank", 0)? as u8;
		let school_tag = node.query_str_opt("scope() > school", 0)?.map(str::to_owned);

		let components = Components::from_kdl(node)?;
		let casting_time = node.query_req_t::<CastingTime>("scope() > casting-time")?;
		let range = node.query_req_t::<Range>("scope() > range")?;
		let area_of_effect = node.query_opt_t::<AreaOfEffect>("scope() > area_of_effect")?;
		let duration = node.query_req_t::<Duration>("scope() > duration")?;
		let check = node.query_opt_t::<Check>("scope() > check")?;
		let damage = node.query_opt_t::<Damage>("scope() > damage")?;

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

		node.entry(("name", self.name.clone()));
		node.child(("source", &self.id, OmitIfEmpty));

		node.child(("school", &self.school_tag, OmitIfEmpty));
		node.children(("tag", self.tags.iter(), OmitIfEmpty));
		node.child(("rank", &self.rank));
		node.child(("casting-time", &self.casting_time));
		node.child(("range", &self.range));
		node.child(("area_of_effect", &self.area_of_effect));
		node += self.components.as_kdl();
		node.child(("duration", &self.duration));
		node.child(("check", &self.check));
		node.child(("damage", &self.damage));
		node.child(("description", &self.description, OmitIfEmpty));

		node
	}
}
