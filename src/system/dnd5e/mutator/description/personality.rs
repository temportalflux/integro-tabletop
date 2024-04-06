use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::data::{
		character::{Character, PersonalityKind},
		description,
	},
	system::Mutator,
	utility::NotInList,
};
use kdlize::OmitIfEmpty;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct SuggestedPersonality {
	kind: PersonalityKind,
	options: Vec<String>,
}

crate::impl_trait_eq!(SuggestedPersonality);
kdlize::impl_kdl_node!(SuggestedPersonality, "suggested_personality");

impl Mutator for SuggestedPersonality {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: SuggestedPersonality description
		description::Section {
			content: Default::default(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, _parent: &ReferencePath) {
		let suggestions = &mut stats.derived_description_mut().personality_suggestions;
		suggestions[self.kind].extend(self.options.clone().into_iter());
	}
}

impl FromKdl<NodeContext> for SuggestedPersonality {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = match node.next_str_req()? {
			"Trait" => PersonalityKind::Trait,
			"Ideal" => PersonalityKind::Ideal,
			"Bond" => PersonalityKind::Bond,
			"Flaw" => PersonalityKind::Flaw,
			name => return Err(NotInList(name.into(), vec!["Trait", "Ideal", "Bond", "Flaw"]).into()),
		};
		let options = node.query_str_all("scope() > option", 0)?;
		let options = options.into_iter().map(str::to_owned).collect::<Vec<_>>();
		Ok(Self { kind, options })
	}
}

impl AsKdl for SuggestedPersonality {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(match &self.kind {
			PersonalityKind::Trait => "Trait",
			PersonalityKind::Ideal => "Ideal",
			PersonalityKind::Bond => "Bond",
			PersonalityKind::Flaw => "Flaw",
		});
		node.children(("option", self.options.iter(), OmitIfEmpty));
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(SuggestedPersonality);

		#[test]
		fn kind_trait() -> anyhow::Result<()> {
			let doc = "
				|mutator \"suggested_personality\" \"Trait\" {
				|    option \"Some option\"
				|}
			";
			let data = SuggestedPersonality {
				kind: PersonalityKind::Trait,
				options: vec!["Some option".into()],
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn kind_ideal() -> anyhow::Result<()> {
			let doc = "
				|mutator \"suggested_personality\" \"Ideal\" {
				|    option \"Some option\"
				|}
			";
			let data = SuggestedPersonality {
				kind: PersonalityKind::Ideal,
				options: vec!["Some option".into()],
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn kind_bond() -> anyhow::Result<()> {
			let doc = "
				|mutator \"suggested_personality\" \"Bond\" {
				|    option \"Some option\"
				|}
			";
			let data = SuggestedPersonality {
				kind: PersonalityKind::Bond,
				options: vec!["Some option".into()],
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn kind_flaw() -> anyhow::Result<()> {
			let doc = "
				|mutator \"suggested_personality\" \"Flaw\" {
				|    option \"Some option\"
				|}
			";
			let data = SuggestedPersonality {
				kind: PersonalityKind::Flaw,
				options: vec!["Some option".into()],
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
