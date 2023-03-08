use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{
		data::{character::Character, Ability},
		DnD5e, FromKDL, KDLNode,
	},
	utility::Mutator,
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct AddSavingThrowModifier {
	pub ability: Option<Ability>,
	pub target: Option<String>,
}

impl crate::utility::TraitEq for AddSavingThrowModifier {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for AddSavingThrowModifier {
	fn id() -> &'static str {
		"add_saving_throw_modifier"
	}
}

impl Mutator for AddSavingThrowModifier {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats
			.saving_throws_mut()
			.add_modifier(self.ability, self.target.clone(), source);
	}
}

impl FromKDL<DnD5e> for AddSavingThrowModifier {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let ability = match node.get_str_opt("ability")? {
			Some(str) => Some(Ability::from_str(str)?),
			None => None,
		};
		let target = node.get_str_opt("target")?.map(str::to_owned);
		Ok(Self { ability, target })
	}
}

// TODO: Test AddSavingThrowModifier FromKDL

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::{
		character::{Character, Persistent},
		Ability, Feature,
	};

	#[test]
	fn advantage() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddSavingThrowModifier".into(),
				mutators: vec![AddSavingThrowModifier {
					ability: Some(Ability::Wisdom),
					target: Some("Magic".into()),
				}
				.into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		let (_, advantages) = &character.saving_throws()[Ability::Wisdom];
		assert_eq!(
			*advantages,
			vec![(Some("Magic".into()), "AddSavingThrowModifier".into())]
		);
	}
}
