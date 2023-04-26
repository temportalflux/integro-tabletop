use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::{
		core::SourceId,
		dnd5e::data::{action::LimitedUses, character::Character, Ability},
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
	Caster,
	AddSource,
	AddPrepared(Vec<SourceId>, Option<LimitedUses>),
}

impl Mutator for Spellcasting {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some("Spellcasting".into())
	}

	fn description(&self) -> Option<String> {
		None
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
			Operation::Caster => {}
			Operation::AddSource => {}
			Operation::AddPrepared(spell_ids, limited_uses) => {
				stats.spellcasting_mut().add_prepared(
					spell_ids,
					self.ability,
					limited_uses.as_ref(),
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
				let cantrips: Option<()> = match node.query_opt("scope() > cantrips")? {
					None => None,
					Some(node) => {
						let ctx = ctx.next_node();
						let class_name = node.get_str_req("class")?.to_owned();
						let restriction: Option<()> =
							match node.query_opt("scope() > restriction")? {
								None => None,
								Some(node) => {
									let mut ctx = ctx.next_node();
									let mut tags = node
										.query_str_all("scope() > tag", 0)?
										.into_iter()
										.map(str::to_owned)
										.collect::<Vec<_>>();
									None
								}
							};
						let mut levels = BTreeMap::new();
						for node in node.query_all("scope() > level")? {
							let mut ctx = ctx.next_node();
							let level = node.get_i64_req(ctx.consume_idx())? as u32;
							let capacity = node.get_i64_req(ctx.consume_idx())? as u32;
							levels.insert(level, capacity);
						}
						None
					}
				};

				Operation::Caster
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
