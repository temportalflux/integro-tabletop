use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::data::{
		character::{Character, StartingEquipment},
		description,
	},
	system::Mutator,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub struct AddStartingEquipment(Vec<StartingEquipment>);

crate::impl_trait_eq!(AddStartingEquipment);
kdlize::impl_kdl_node!(AddStartingEquipment, "add_starting_equipment");

impl Mutator for AddStartingEquipment {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section { ..Default::default() }
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		stats.add_starting_equipment(&self.0, parent);
	}
}

impl FromKdl<NodeContext> for AddStartingEquipment {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(Self(StartingEquipment::from_kdl_vec(node)?))
	}
}
impl AsKdl for AddStartingEquipment {
	fn as_kdl(&self) -> NodeBuilder {
		StartingEquipment::to_kdl_vec(&self.0)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::{
				dnd5e::{
					data::{
						character::IndirectItem,
						currency::{self, Wallet},
						item::{restriction, weapon, Item, Restriction},
					},
					mutator::test::test_utils,
				},
				SourceId,
			},
		};

		test_utils!(AddStartingEquipment);

		#[test]
		fn item_specific() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Specific\" \"items/weapons/rapier.kdl\"
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::IndirectItem(IndirectItem::Specific(
				SourceId {
					path: "items/weapons/rapier.kdl".into(),
					..Default::default()
				},
				1,
			))]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn item_custom() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Custom\" name=\"Trophy\" {
				|        description \"trophy taken from a fallen enemy\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::IndirectItem(IndirectItem::Custom(Item {
				name: "Trophy".into(),
				description: description::Info {
					sections: vec![description::Section {
						content: description::SectionContent::Body("trophy taken from a fallen enemy".into()),
						..Default::default()
					}],
					..Default::default()
				},
				..Default::default()
			}))]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn item_select_tagged() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Select\" {
				|        tag \"Arcane Focus\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::SelectItem(Restriction {
				tags: vec!["Arcane Focus".into()],
				..Default::default()
			})]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn item_select_weapon() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Select\" {
				|        weapon kind=\"Simple\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::SelectItem(Restriction {
				weapon: Some(restriction::Weapon {
					kind: Some(weapon::Kind::Simple),
					..Default::default()
				}),
				..Default::default()
			})]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn item_select_weapon_melee() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Select\" {
				|        weapon kind=\"Martial\" has_melee=true
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::SelectItem(Restriction {
				weapon: Some(restriction::Weapon {
					kind: Some(weapon::Kind::Martial),
					has_melee: Some(true),
					..Default::default()
				}),
				..Default::default()
			})]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn currency() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    currency 15 (Currency)\"Gold\"
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::Currency(Wallet::from([(
				15,
				currency::Kind::Gold,
			)]))]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn pick_one() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    group pick=1 {
				|        item \"Specific\" \"items/weapons/rapier.kdl\"
				|        item \"Specific\" \"items/weapons/longsword.kdl\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::Group {
				entries: vec![
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapons/rapier.kdl".into(),
							..Default::default()
						},
						1,
					)),
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapons/longsword.kdl".into(),
							..Default::default()
						},
						1,
					)),
				],
				pick: Some(1),
			}]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn group() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    group {
				|        item \"Specific\" \"items/weapons/shortsword.kdl\"
				|        item \"Specific\" \"items/weapons/shortsword.kdl\"
				|        item \"Specific\" \"items/weapons/longbow.kdl\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::Group {
				entries: vec![
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapons/shortsword.kdl".into(),
							..Default::default()
						},
						1,
					)),
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapons/shortsword.kdl".into(),
							..Default::default()
						},
						1,
					)),
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapons/longbow.kdl".into(),
							..Default::default()
						},
						1,
					)),
				],
				pick: None,
			}]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
