use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	system::dnd5e::data::{
		character::{Character, StartingEquipment},
		description,
	},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddStartingEquipment(Vec<StartingEquipment>);

crate::impl_trait_eq!(AddStartingEquipment);
crate::impl_kdl_node!(AddStartingEquipment, "add_starting_equipment");

impl Mutator for AddStartingEquipment {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		stats.add_starting_equipment(&self.0, parent);
	}
}

impl FromKDL for AddStartingEquipment {
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
				core::SourceId,
				dnd5e::{
					data::{
						character::{IndirectItem, ItemFilter, WeaponFilter},
						currency::{self, Wallet},
						item::{weapon, Item},
					},
					mutator::test::test_utils,
				},
			},
		};

		test_utils!(AddStartingEquipment);

		#[test]
		fn item_specific() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Specific\" \"items/weapon/rapier.kdl\"
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::IndirectItem(
				IndirectItem::Specific(
					SourceId {
						path: "items/weapon/rapier.kdl".into(),
						..Default::default()
					},
					1,
				),
			)]);
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
			let data = AddStartingEquipment(vec![StartingEquipment::IndirectItem(
				IndirectItem::Custom(Item {
					name: "Trophy".into(),
					description: description::Info {
						sections: vec![description::Section {
							content: description::SectionContent::Body(
								"trophy taken from a fallen enemy".into(),
							),
							..Default::default()
						}],
						..Default::default()
					},
					..Default::default()
				}),
			)]);
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
			let data = AddStartingEquipment(vec![StartingEquipment::SelectItem(ItemFilter {
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
			let data = AddStartingEquipment(vec![StartingEquipment::SelectItem(ItemFilter {
				weapon: Some(WeaponFilter {
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
			let data = AddStartingEquipment(vec![StartingEquipment::SelectItem(ItemFilter {
				weapon: Some(WeaponFilter {
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
				|        item \"Specific\" \"items/weapon/rapier.kdl\"
				|        item \"Specific\" \"items/weapon/longsword.kdl\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![StartingEquipment::Group {
				entries: vec![
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapon/rapier.kdl".into(),
							..Default::default()
						},
						1,
					)),
					StartingEquipment::IndirectItem(IndirectItem::Specific(
						SourceId {
							path: "items/weapon/longsword.kdl".into(),
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
