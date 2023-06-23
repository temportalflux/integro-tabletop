use std::str::FromStr;

use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeExt},
	system::{
		core::SourceId,
		dnd5e::data::{
			character::Character,
			currency::Wallet,
			description,
			item::{weapon, Item},
		},
	},
	utility::{Mutator, NotInList},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddStartingEquipment(Vec<Entry>);

#[derive(Clone, Debug, PartialEq)]
enum Entry {
	Currency(Wallet),
	SpecificItem(SourceId),
	CustomItem(Item),
	SelectItem(ItemFilter),
	PickOne(Vec<Entry>),
	Group(Vec<Entry>),
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ItemFilter {
	tags: Vec<String>,
	weapon: Option<WeaponFilter>,
}
#[derive(Clone, Debug, PartialEq, Default)]
struct WeaponFilter {
	kind: Option<weapon::Kind>,
	has_melee: Option<bool>,
}

crate::impl_trait_eq!(AddStartingEquipment);
crate::impl_kdl_node!(AddStartingEquipment, "add_starting_equipment");

impl Mutator for AddStartingEquipment {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			..Default::default()
		}
	}

	fn apply(&self, _stats: &mut Character, _parent: &std::path::Path) {
		// TODO: apply starting equipment data
	}
}

fn entries_from_kdl(
	node: &kdl::KdlNode,
	ctx: &mut crate::kdl_ext::NodeContext,
) -> anyhow::Result<Vec<Entry>> {
	let mut entries = Vec::new();
	if let Some(children) = node.children() {
		for node in children.nodes() {
			entries.push(Entry::from_kdl(node, &mut ctx.next_node())?);
		}
	}
	Ok(entries)
}
fn entries_to_kdl(entries: &Vec<Entry>) -> NodeBuilder {
	let mut node = NodeBuilder::default();
	for entry in entries {
		node.push_child_t(entry.node_name(), entry);
	}
	node
}

impl FromKDL for AddStartingEquipment {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(entries_from_kdl(node, ctx)?))
	}
}
impl AsKdl for AddStartingEquipment {
	fn as_kdl(&self) -> NodeBuilder {
		entries_to_kdl(&self.0)
	}
}

impl Entry {
	fn node_name(&self) -> &'static str {
		match self {
			Self::Currency(_) => "currency",
			Self::SpecificItem(_) | Self::CustomItem(_) | Self::SelectItem(_) => "item",
			Self::PickOne(_) => "pick-one",
			Self::Group(_) => "group",
		}
	}
}
impl FromKDL for Entry {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.name().value() {
			"currency" => Ok(Self::Currency(Wallet::from_kdl(node, ctx)?)),
			"pick-one" => Ok(Self::PickOne(entries_from_kdl(node, ctx)?)),
			"group" => Ok(Self::Group(entries_from_kdl(node, ctx)?)),
			"item" => match node.get_str_req(ctx.consume_idx())? {
				"Specific" => {
					let id = SourceId::from_str(node.get_str_req(ctx.consume_idx())?)?;
					Ok(Self::SpecificItem(id.with_basis(ctx.id(), false)))
				}
				"Custom" => Ok(Self::CustomItem(Item::from_kdl(node, ctx)?)),
				"Select" => Ok(Self::SelectItem(ItemFilter::from_kdl(node, ctx)?)),
				kind => Err(NotInList(kind.into(), vec!["Specific", "Custom", "Select"]).into()),
			},
			name => {
				Err(NotInList(name.into(), vec!["item", "currency", "pick-one", "group"]).into())
			}
		}
	}
}
impl AsKdl for Entry {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Currency(wallet) => wallet.as_kdl(),
			Self::SpecificItem(id) => NodeBuilder::default()
				.with_entry("Specific")
				.with_entry(id.to_string()),
			Self::CustomItem(item) => NodeBuilder::default()
				.with_entry("Custom")
				.with_extension(item.as_kdl()),
			Self::SelectItem(filter) => NodeBuilder::default()
				.with_entry("Select")
				.with_extension(filter.as_kdl()),
			Self::PickOne(entries) => entries_to_kdl(entries),
			Self::Group(entries) => entries_to_kdl(entries),
		}
	}
}

impl FromKDL for ItemFilter {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let tags = node
			.query_str_all("scope() > tag", 0)?
			.into_iter()
			.map(str::to_owned)
			.collect::<Vec<_>>();
		let weapon = match node.query_opt("scope() > weapon")? {
			None => None,
			Some(node) => Some(WeaponFilter::from_kdl(node, &mut ctx.next_node())?),
		};
		Ok(Self { tags, weapon })
	}
}
impl AsKdl for ItemFilter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for tag in &self.tags {
			node.push_child_t("tag", tag);
		}
		if let Some(weapon_filter) = &self.weapon {
			node.push_child_t("weapon", weapon_filter);
		}
		node
	}
}

impl FromKDL for WeaponFilter {
	fn from_kdl(
		node: &kdl::KdlNode,
		_ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let kind = match node.get_str_opt("kind")? {
			None => None,
			Some(str) => Some(weapon::Kind::from_str(str)?),
		};
		let has_melee = node.get_bool_opt("has_melee")?;
		Ok(Self { kind, has_melee })
	}
}
impl AsKdl for WeaponFilter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(kind) = &self.kind {
			node.push_entry(("kind", kind.to_string()));
		}
		if let Some(has_melee) = &self.has_melee {
			node.push_entry(("has_melee", *has_melee));
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::dnd5e::{data::currency, mutator::test::test_utils},
		};

		test_utils!(AddStartingEquipment);

		#[test]
		fn item_specific() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_starting_equipment\" {
				|    item \"Specific\" \"items/weapon/rapier.kdl\"
				|}
			";
			let data = AddStartingEquipment(vec![Entry::SpecificItem(SourceId {
				path: "items/weapon/rapier.kdl".into(),
				..Default::default()
			})]);
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
			let data = AddStartingEquipment(vec![Entry::CustomItem(Item {
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
			})]);
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
			let data = AddStartingEquipment(vec![Entry::SelectItem(ItemFilter {
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
			let data = AddStartingEquipment(vec![Entry::SelectItem(ItemFilter {
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
			let data = AddStartingEquipment(vec![Entry::SelectItem(ItemFilter {
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
			let data = AddStartingEquipment(vec![Entry::Currency(Wallet::from([(
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
				|    pick-one {
				|        item \"Specific\" \"items/weapon/rapier.kdl\"
				|        item \"Specific\" \"items/weapon/longsword.kdl\"
				|    }
				|}
			";
			let data = AddStartingEquipment(vec![Entry::PickOne(vec![
				Entry::SpecificItem(SourceId {
					path: "items/weapon/rapier.kdl".into(),
					..Default::default()
				}),
				Entry::SpecificItem(SourceId {
					path: "items/weapon/longsword.kdl".into(),
					..Default::default()
				}),
			])]);
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
			let data = AddStartingEquipment(vec![Entry::Group(vec![
				Entry::SpecificItem(SourceId {
					path: "items/weapons/shortsword.kdl".into(),
					..Default::default()
				}),
				Entry::SpecificItem(SourceId {
					path: "items/weapons/shortsword.kdl".into(),
					..Default::default()
				}),
				Entry::SpecificItem(SourceId {
					path: "items/weapons/longbow.kdl".into(),
					..Default::default()
				}),
			])]);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
