use crate::{
	kdl_ext::NodeContext,
	system::{dnd5e::data::character::Character, SourceId},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasSpell {
	spell_id: SourceId,
}

crate::impl_trait_eq!(HasSpell);
kdlize::impl_kdl_node!(HasSpell, "has_spell");

impl crate::system::Evaluator for HasSpell {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(format!("has spell (full id) {}", self.spell_id.to_string()))
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		// Check the list of always preparred spells (those granted by class features and are not selected by users)
		if character.spellcasting().prepared_spells().contains_key(&self.spell_id) {
			return Ok(());
		}

		// Check the list of spells that were selected by the user
		let mut iter_selected_spells = character.selected_spells().iter_selected();
		if let Some(_) = iter_selected_spells.find(|(_caster, spell_id, _spell)| *spell_id == &self.spell_id) {
			return Ok(());
		}

		Err(format!("spell is not known or preparred"))
	}
}

impl FromKdl<NodeContext> for HasSpell {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let spell_id = node.next_str_req_t::<SourceId>()?;
		let spell_id = spell_id.unversioned();
		Ok(Self { spell_id })
	}
}

impl AsKdl for HasSpell {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.spell_id.unversioned().to_string().as_str());
		node
	}
}
