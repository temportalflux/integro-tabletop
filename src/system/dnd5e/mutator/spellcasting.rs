use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::{
		core::SourceId,
		dnd5e::data::{
			action::LimitedUses,
			character::{
				spellcasting::{Caster, Restriction, Slots, SpellCapacity},
				Character,
			},
			description, Ability,
		},
	},
	utility::{Mutator, NotInList},
};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub struct Spellcasting {
	ability: Ability,
	operation: Operation,
}

crate::impl_trait_eq!(Spellcasting);
crate::impl_kdl_node!(Spellcasting, "spellcasting");

#[derive(Clone, Debug, PartialEq)]
enum Operation {
	Caster(Caster),
	AddSource,
	AddPrepared(Vec<SourceId>, Option<LimitedUses>),
}

impl Mutator for Spellcasting {
	type Target = Character;

	fn description(&self) -> description::Section {
		description::Section {
			title: Some("Spellcasting".into()),
			content: format!("{:?}", self),
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		match &self.operation {
			Operation::AddPrepared(_, Some(limited_uses)) => {
				limited_uses.set_data_path(parent);
			}
			_ => {}
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		match &self.operation {
			Operation::Caster(caster) => {
				stats.spellcasting_mut().add_caster(caster.clone());
			}
			Operation::AddSource => {}
			Operation::AddPrepared(spell_ids, limited_uses) => {
				stats.spellcasting_mut().add_prepared(
					spell_ids,
					self.ability,
					limited_uses.as_ref(),
					parent,
				);
			}
		}
	}
}

impl FromKDL for Spellcasting {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let ability = Ability::from_str(node.get_str_req("ability")?)?;
		let operation = match node.get_str_opt(ctx.consume_idx())? {
			None => {
				let class_name = node.get_str_req("class")?.to_owned();
				let restriction = {
					let node = node.query_req("scope() > restriction")?;
					let _ctx = ctx.next_node();
					let tags = node
						.query_str_all("scope() > tag", 0)?
						.into_iter()
						.map(str::to_owned)
						.collect::<Vec<_>>();
					Restriction { tags }
				};

				let cantrip_capacity = match node.query_opt("scope() > cantrips")? {
					None => None,
					Some(node) => {
						let ctx = ctx.next_node();

						let mut level_map = BTreeMap::new();
						for node in node.query_all("scope() > level")? {
							let mut ctx = ctx.next_node();
							let level = node.get_i64_req(ctx.consume_idx())? as usize;
							let capacity = node.get_i64_req(ctx.consume_idx())? as usize;
							level_map.insert(level, capacity);
						}

						Some(level_map)
					}
				};

				let slots =
					Slots::from_kdl(node.query_req("scope() > slots")?, &mut ctx.next_node())?;

				let spell_capacity = {
					let node = node.query_req("scope() > kind")?;
					let mut ctx = ctx.next_node();
					match node.get_str_req(ctx.consume_idx())? {
						"Prepared" => {
							let capacity = {
								let node = node.query_req("scope() > capacity")?;
								ctx.parse_evaluator::<Character, i32>(node)?
							};
							SpellCapacity::Prepared(capacity)
						}
						"Known" => {
							let capacity = {
								let node = node.query_req("scope() > capacity")?;
								let ctx = ctx.next_node();
								let mut capacity = BTreeMap::new();
								for node in node.query_all("scope() > level")? {
									let mut ctx = ctx.next_node();
									let level = node.get_i64_req(ctx.consume_idx())? as usize;
									let amount = node.get_i64_req(ctx.consume_idx())? as usize;
									capacity.insert(level, amount);
								}
								capacity
							};
							SpellCapacity::Known(capacity)
						}
						name => {
							return Err(NotInList(name.into(), vec!["Known", "Prepared"]).into());
						}
					}
				};

				Operation::Caster(Caster {
					class_name,
					ability,
					restriction,
					cantrip_capacity,
					slots,
					spell_capacity,
				})
			}
			Some("add_source") => {
				let mut spells = Vec::new();
				for s in node.query_str_all("scope() > spell", 0)? {
					spells.push(SourceId::from_str(s)?.with_basis(ctx.id()));
				}
				Operation::AddSource
			}
			Some("add_prepared") => {
				let mut spells = Vec::new();
				for s in node.query_str_all("scope() > spell", 0)? {
					spells.push(SourceId::from_str(s)?.with_basis(ctx.id()));
				}
				let limited_uses = match node.query_opt("scope() > limited_use")? {
					None => None,
					Some(node) => Some(LimitedUses::from_kdl(node, &mut ctx.next_node())?),
				};
				Operation::AddPrepared(spells, limited_uses)
			}
			Some(name) => {
				return Err(NotInList(name.into(), vec!["add_source", "add_prepared"]).into())
			}
		};
		Ok(Self { ability, operation })
	}
}
