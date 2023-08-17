use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::data::{action::ActivationKind, Indirect, Spell},
	},
};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SpellContainer {
	can_transcribe_from: bool,
	can_prepare_from: bool,
	capacity: Capacity,
	casting: Option<Casting>,
	spells: Vec<ContainerSpell>,
}

// Describes how many spells and of what types this container can hold.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Capacity {
	// How many individual spells can be contained.
	max_count: Option<usize>,
	// The minimum allowed rank of any individual spell.
	rank_min: Option<u8>,
	// The maximum allowed rank of any individual spell.
	rank_max: Option<u8>,
	// The max total value allowed when the ranks of all contained spells are summed.
	rank_total: Option<usize>,
}

// Describes the conditions under which all spells are cast, unless overriden on a given entry.
#[derive(Clone, PartialEq, Debug)]
pub struct Casting {
	// What kind of action is used to cast from this container.
	// If not provided, the activation is defined by the spell itself.
	activation_kind: Option<ActivationKind>,
	// If casting tue last spell in the container will consume the item (destroy it).
	// If the spell is transcribed from this container and this property is enabled,
	// the item is destroyed (TODO: transcribing spells is not a feature yet).
	consume_item: bool,
	// If casting any spell from the container will consume the spell in the container.
	consume_spell: bool,
	// What the save DC is for a spell cast from the container.
	save_dc: Option<u8>,
	// What the attack bonus is for a spell cast from the container.
	attack_bonus: Option<i32>,
}

// A spell which is stored in a Spell Container.
#[derive(Clone, PartialEq, Debug)]
pub struct ContainerSpell {
	spell: Indirect<Spell>,
	// The overridden rank of the spell. If provided, the spell must be cast at this rank.
	rank: Option<u8>,
	// The spell save DC that must be used for this spell.
	save_dc: Option<u8>,
	// The spell attack bonus that must be used for this spell.
	attack_bonus: Option<i32>,
}

impl FromKDL for SpellContainer {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let can_transcribe_from = node.get_bool_opt("transcribe")?.unwrap_or_default();
		let can_prepare_from = node.get_bool_opt("prepare_from")?.unwrap_or_default();
		let capacity = node
			.query_opt_t::<Capacity>("scope() > capacity")?
			.unwrap_or_default();
		let casting = node.query_opt_t::<Casting>("scope() > casting")?;
		let spells = node.query_all_t::<ContainerSpell>("scope() > spell")?;
		Ok(Self {
			can_transcribe_from,
			can_prepare_from,
			capacity,
			casting,
			spells,
		})
	}
}
impl AsKdl for SpellContainer {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if self.can_transcribe_from {
			node.push_entry(("transcribe", true));
		}
		if self.can_prepare_from {
			node.push_entry(("prepare_from", true));
		}

		node.push_child_opt_t("capacity", &self.capacity);
		if let Some(casting) = &self.casting {
			node.push_child_t("casting", casting);
		}

		for container_spell in &self.spells {
			node.push_child_t("spell", container_spell);
		}

		node
	}
}

impl FromKDL for Capacity {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let max_count = node.next_i64_opt()?.map(|v| v as usize);
		let rank_min = node
			.query_i64_opt("scope() > rank", "min")?
			.map(|v| v as u8);
		let rank_max = node
			.query_i64_opt("scope() > rank", "max")?
			.map(|v| v as u8);
		let rank_total = node
			.query_i64_opt("scope() > rank", "total")?
			.map(|v| v as usize);
		Ok(Self {
			max_count,
			rank_min,
			rank_max,
			rank_total,
		})
	}
}
impl AsKdl for Capacity {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if let Some(num) = &self.max_count {
			node.push_entry(*num as i64);
		}

		node.push_child_opt({
			let mut node = NodeBuilder::default();
			if let Some(num) = &self.rank_min {
				node.push_entry(("min", *num as i64));
			}
			if let Some(num) = &self.rank_max {
				node.push_entry(("max", *num as i64));
			}
			if let Some(num) = &self.rank_total {
				node.push_entry(("total", *num as i64));
			}
			node.build("rank")
		});

		node
	}
}

impl FromKDL for Casting {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let activation_kind = node.next_str_opt_t()?;
		let consume_item = node
			.query_bool_opt("scope() > consume_item", 0)?
			.unwrap_or_default();
		let consume_spell = node
			.query_bool_opt("scope() > consume_spell", 0)?
			.unwrap_or_default();
		let save_dc = node.query_i64_opt("scope() > save_dc", 0)?.map(|v| v as u8);
		let attack_bonus = node
			.query_i64_opt("scope() > atk_bonus", 0)?
			.map(|v| v as i32);
		Ok(Self {
			activation_kind,
			consume_item,
			consume_spell,
			save_dc,
			attack_bonus,
		})
	}
}
impl AsKdl for Casting {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if let Some(kind) = &self.activation_kind {
			node += kind.as_kdl();
		}

		if self.consume_item {
			node.push_child_entry("consume_item", true);
		}
		if self.consume_spell {
			node.push_child_entry("consume_spell", true);
		}

		if let Some(num) = &self.save_dc {
			node.push_child_t("save_dc", num);
		}
		if let Some(num) = &self.attack_bonus {
			node.push_child_t("atk_bonus", num);
		}

		node
	}
}

impl From<SourceId> for ContainerSpell {
	fn from(id: SourceId) -> Self {
		Self {
			spell: Indirect::Id(id),
			rank: None,
			save_dc: None,
			attack_bonus: None,
		}
	}
}

impl FromKDL for ContainerSpell {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let spell = Indirect::from_kdl(node)?;
		let rank = node.get_i64_opt("rank")?.map(|v| v as u8);
		let save_dc = node.get_i64_opt("save_dc")?.map(|v| v as u8);
		let attack_bonus = node.get_i64_opt("atk_bonus")?.map(|v| v as i32);
		Ok(Self {
			spell,
			rank,
			save_dc,
			attack_bonus,
		})
	}
}
impl AsKdl for ContainerSpell {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.spell.as_kdl();

		if let Some(num) = &self.rank {
			node.push_entry(("rank", *num as i64));
		}

		if let Some(num) = &self.save_dc {
			node.push_entry(("save_dc", *num as i64));
		}
		if let Some(num) = &self.attack_bonus {
			node.push_entry(("atk_bonus", *num as i64));
		}

		node
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use super::*;
	use crate::kdl_ext::test_utils::*;

	static NODE_NAME: &str = "spells";

	#[test]
	fn empty_spellbook() -> anyhow::Result<()> {
		let doc = "
			|spells prepare_from=true
		";
		let data = SpellContainer {
			can_prepare_from: true,
			..Default::default()
		};
		assert_eq_fromkdl!(SpellContainer, doc, data);
		assert_eq_askdl!(&data, doc);
		Ok(())
	}

	#[test]
	fn looted_spellbook() -> anyhow::Result<()> {
		let doc = "
			|spells transcribe=true {
			|    spell \"local://basic-rules@dnd5e/spells/arcaneLock.kdl\"
			|    spell \"local://basic-rules@dnd5e/spells/glyphOfWarding.kdl\"
			|    spell \"local://basic-rules@dnd5e/spells/identify.kdl\"
			|    spell \"local://basic-rules@dnd5e/spells/falseLife.kdl\"
			|    spell \"local://basic-rules@dnd5e/spells/teleport.kdl\"
			|}
		";
		let data = SpellContainer {
			can_transcribe_from: true,
			spells: vec![
				ContainerSpell::from(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/arcaneLock.kdl",
				)?),
				ContainerSpell::from(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/glyphOfWarding.kdl",
				)?),
				ContainerSpell::from(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/identify.kdl",
				)?),
				ContainerSpell::from(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/falseLife.kdl",
				)?),
				ContainerSpell::from(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/teleport.kdl",
				)?),
			],
			..Default::default()
		};
		assert_eq_fromkdl!(SpellContainer, doc, data);
		assert_eq_askdl!(&data, doc);
		Ok(())
	}

	#[test]
	fn container_with_capacity() -> anyhow::Result<()> {
		let doc = "
			|spells {
			|    capacity {
			|        rank min=1 max=5 total=5
			|    }
			|    spell \"local://basic-rules@dnd5e/spells/identify.kdl\" rank=2 save_dc=15 atk_bonus=2
			|}
		";
		let data = SpellContainer {
			capacity: Capacity {
				max_count: None,
				rank_min: Some(1),
				rank_max: Some(5),
				rank_total: Some(5),
			},
			spells: vec![ContainerSpell {
				spell: Indirect::Id(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/identify.kdl",
				)?),
				rank: Some(2),
				save_dc: Some(15),
				attack_bonus: Some(2),
			}],
			..Default::default()
		};
		assert_eq_fromkdl!(SpellContainer, doc, data);
		assert_eq_askdl!(&data, doc);
		Ok(())
	}

	#[test]
	fn single_use() -> anyhow::Result<()> {
		let doc = "
			|spells {
			|    capacity 1 {
			|        rank min=3 max=3
			|    }
			|    casting {
			|        consume_item true
			|        save_dc 13
			|        atk_bonus 5
			|    }
			|    spell \"local://basic-rules@dnd5e/spells/fireball.kdl\"
			|}
		";
		let data = SpellContainer {
			capacity: Capacity {
				max_count: Some(1),
				rank_min: Some(3),
				rank_max: Some(3),
				rank_total: None,
			},
			casting: Some(Casting {
				activation_kind: None,
				consume_item: true,
				consume_spell: false,
				save_dc: Some(13),
				attack_bonus: Some(5),
			}),
			spells: vec![ContainerSpell {
				spell: Indirect::Id(SourceId::from_str(
					"local://basic-rules@dnd5e/spells/fireball.kdl",
				)?),
				rank: None,
				save_dc: None,
				attack_bonus: None,
			}],
			..Default::default()
		};
		assert_eq_fromkdl!(SpellContainer, doc, data);
		assert_eq_askdl!(&data, doc);
		Ok(())
	}

	#[test]
	fn refillable() -> anyhow::Result<()> {
		let doc = "
			|spells {
			|    capacity 1 {
			|        rank max=0
			|    }
			|    casting \"Action\" {
			|        consume_spell true
			|    }
			|}
		";
		let data = SpellContainer {
			capacity: Capacity {
				max_count: Some(1),
				rank_min: None,
				rank_max: Some(0),
				rank_total: None,
			},
			casting: Some(Casting {
				activation_kind: Some(ActivationKind::Action),
				consume_item: false,
				consume_spell: true,
				save_dc: None,
				attack_bonus: None,
			}),
			..Default::default()
		};
		assert_eq_fromkdl!(SpellContainer, doc, data);
		assert_eq_askdl!(&data, doc);
		Ok(())
	}
}
