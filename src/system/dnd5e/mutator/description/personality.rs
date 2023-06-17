use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{
		character::{Character, PersonalityKind},
		description,
	},
	utility::{Mutator, NotInList},
};

#[derive(Clone, PartialEq, Debug)]
pub struct SuggestedPersonality {
	kind: PersonalityKind,
	options: Vec<String>,
}

crate::impl_trait_eq!(SuggestedPersonality);
crate::impl_kdl_node!(SuggestedPersonality, "suggested_personality");

impl Mutator for SuggestedPersonality {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: SuggestedPersonality description
		description::Section {
			content: Default::default(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		let suggestions = &mut stats.derived_description_mut().personality_suggestions;
		suggestions[self.kind].extend(self.options.clone().into_iter());
	}
}

impl FromKDL for SuggestedPersonality {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let kind = match node.get_str_req(ctx.consume_idx())? {
			"Trait" => PersonalityKind::Trait,
			"Ideal" => PersonalityKind::Ideal,
			"Bond" => PersonalityKind::Bond,
			"Flaw" => PersonalityKind::Flaw,
			name => {
				return Err(NotInList(name.into(), vec!["Trait", "Ideal", "Bond", "Flaw"]).into())
			}
		};
		let options = node.query_str_all("scope() > option", 0)?;
		let options = options.into_iter().map(str::to_owned).collect::<Vec<_>>();
		Ok(Self { kind, options })
	}
}
// TODO AsKdl: from/as tests for SuggestedPersonality
impl AsKdl for SuggestedPersonality {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(match &self.kind {
			PersonalityKind::Trait => "Trait",
			PersonalityKind::Ideal => "Ideal",
			PersonalityKind::Bond => "Bond",
			PersonalityKind::Flaw => "Flaw",
		});
		for option in &self.options {
			node.push_child_opt_t("option", option);
		}
		node
	}
}
