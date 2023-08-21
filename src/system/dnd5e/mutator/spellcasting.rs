use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, KDLNode, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::data::{
			action::LimitedUses,
			character::{
				spellcasting::{self, Caster, Restriction, RitualCapability, Slots, SpellEntry},
				Character,
			},
			description,
			roll::Roll,
			spell::{self, Spell},
			Ability,
		},
	},
	utility::{selector, Mutator, NotInList, Value},
};
use itertools::Itertools;
use std::{
	collections::{BTreeMap, HashSet},
	str::FromStr,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Spellcasting(Operation);

crate::impl_trait_eq!(Spellcasting);
crate::impl_kdl_node!(Spellcasting, "spellcasting");

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
	Caster(Caster),
	/// Spells added to the list of spells that a caster can know or prepare.
	/// These DO count against the character's known/prepared spell capacity limits,
	/// because the user has to select them to know/prepare them (this just makes the spells available to be selected).
	AddSource {
		class_name: String,
		spell_ids: Vec<SourceId>,
	},
	/// Spells that are always available to be cast, and
	/// DO NOT count against the character's known/prepared spell capacity limits.
	AddPrepared {
		ability: Ability,
		/// TODO: prepared spells can be treated as spells for a given class
		/// If provided, the specified spells are treated as tho they were prepared using this caster class.
		classified_as: Option<String>,
		/// The spells this feature provides, with any additional metadata.
		specific_spells: Vec<(SourceId, PreparedInfo)>,
		selectable_spells: Option<SelectableSpells>,
		/// If provided, the specified spells are cast by using a specific usage criteria.
		/// If a provided spell also allows it to be cast through a slot, then both methods are valid.
		/// Otherwise, if both this is None and it cannot be cast via a slot, then the spell is cast at-will.
		limited_uses: Option<LimitedUses>,
	},
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PreparedInfo {
	/// If the spell can be cast using a spell slot.
	/// If false, the spell is either cast At-Will or through a LimitedUses.
	pub can_cast_through_slot: bool,
	// If the spell can be cast as a ritual.
	pub can_ritual_cast: bool,
	/// If present, the spell must be cast using the specified casting range.
	pub range: Option<spell::Range>,
	/// If present, the spell can only be cast at this rank using this feature.
	pub cast_at_rank: Option<u8>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SelectableSpells {
	pub filter: Option<spellcasting::Filter>,
	/// For all prepared spells which allow the user to select them, this is the selector that is used.
	pub selector: selector::Object<Character>,
	pub prepared: PreparedInfo,
}

impl Mutator for Spellcasting {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		match &self.0 {
			Operation::Caster(caster) => description::Section {
				title: Some("Spellcasting".into()),
				content: format!(
					"Cast spells as a {} using {}.",
					caster.name(),
					caster.ability.long_name()
				)
				.into(),
				..Default::default()
			},
			Operation::AddPrepared {
				ability,
				selectable_spells,
				..
			} => {
				let mut selectors = selector::DataList::default();
				if let Some(selectable) = selectable_spells {
					selectors =
						selectors.with_object("Selected Spells", &selectable.selector, state);
				}
				description::Section {
					title: Some("Spellcasting: Always Preppared Spells".into()),
					content: format!(
						"Add spells which are always prepared, using {}.",
						ability.long_name()
					)
					.into(),
					children: vec![selectors.into()],
					..Default::default()
				}
			}
			Operation::AddSource {
				class_name,
				spell_ids,
			} => description::Section {
				title: Some("Spellcasting: Expanded Spell List".into()),
				content: format!(
					"Adds {} spells you can select from for the {class_name} class.",
					spell_ids.len()
				)
				.into(),
				..Default::default()
			},
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		match &self.0 {
			Operation::AddPrepared {
				selectable_spells,
				limited_uses,
				..
			} => {
				if let Some(selectable_spells) = selectable_spells {
					selectable_spells.selector.set_data_path(parent);
				}
				if let Some(limited_uses) = limited_uses {
					limited_uses.set_data_path(parent);
				}
			}
			_ => {}
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		match &self.0 {
			Operation::Caster(caster) => {
				// The reset entry for standard spell slots is taken care of by `Persistent::SelectedSpells`
				// (where the data paths for spell slots are found).
				// We need to add reset entries for all bonus spell slots at the character's current level.
				let current_level = stats.level(Some(&caster.class_name));
				for slots in &caster.bonus_slots {
					let Slots::Bonus { reset_on, slots_capacity } = slots else {
						continue;
					};
					// Since we are compiling the character at a specific level, we only need to
					// submit reset data for the slots granted at that level.
					if let Some(ranks) = slots_capacity.get(&current_level) {
						for (rank, amount) in ranks {
							// Each rank has its own data path and amount of slots granted by this slot-group,
							// so we need to submit a separate reset for it
							// (because restore amount is applied to all data paths in an entry).
							let rank_data_path = stats
								.persistent()
								.selected_spells
								.consumed_slots_path(*rank);
							let entry = crate::system::dnd5e::data::character::RestEntry {
								restore_amount: Some(Roll::from(*amount as u32)),
								data_paths: vec![rank_data_path],
								source: parent.join(format!(
									"{} Spellcasting Slots (Rank {})",
									caster.name(),
									*rank
								)),
							};
							stats.rest_resets_mut().add(*reset_on, entry);
						}
					}
				}
				stats.spellcasting_mut().add_caster(caster.clone());
			}
			Operation::AddSource {
				class_name,
				spell_ids,
			} => {
				stats
					.spellcasting_mut()
					.add_spell_access(class_name, spell_ids, parent);
			}
			Operation::AddPrepared {
				ability,
				classified_as,
				specific_spells,
				selectable_spells,
				limited_uses,
			} => {
				if let Some(uses) = limited_uses.as_ref() {
					if let LimitedUses::Usage(data) = uses {
						stats.features_mut().register_usage(data, parent);
						if let Some(rest) = data.get_reset_rest(stats) {
							if let Some(data_path) = data.get_data_path() {
								let entry = crate::system::dnd5e::data::character::RestEntry {
									restore_amount: None,
									data_paths: vec![data_path],
									source: parent.join("Prepared Spellcasting"),
								};
								stats.rest_resets_mut().add(rest, entry);
							}
						}
					}
				}
				let mut all_spells = specific_spells
					.iter()
					.map(|(id, info)| (id.clone(), info))
					.collect::<Vec<_>>();
				if let Some(selectable) = &selectable_spells {
					if let Some(data_path) = selectable.selector.get_data_path() {
						if let Some(selections) = stats.get_selections_at(&data_path) {
							let ids = selections
								.iter()
								.filter_map(|str| SourceId::from_str(str).ok());
							let ids_info = ids.map(|id| (id, &selectable.prepared));
							all_spells.extend(ids_info);
						}
					}
				}
				for (id, prepared_info) in all_spells {
					let entry = SpellEntry {
						ability: *ability,
						source: parent.to_owned(),
						classified_as: classified_as.clone(),
						cast_via_slot: prepared_info.can_cast_through_slot,
						cast_via_ritual: prepared_info.can_ritual_cast,
						cast_via_uses: limited_uses.clone(),
						range: prepared_info.range.clone(),
						forced_rank: prepared_info.cast_at_rank.clone(),
					};
					stats.spellcasting_mut().add_prepared(&id, entry);
				}
			}
		}
	}
}

impl FromKDL for Spellcasting {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let operation = match node.next_str_opt()? {
			None => {
				let ability = node.get_str_req_t::<Ability>("ability")?;
				let class_name = node.get_str_req("class")?.to_owned();
				let restriction = {
					let node = node.query_req("scope() > restriction")?;
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
						let mut level_map = BTreeMap::new();
						for node in &mut node.query_all("scope() > level")? {
							let level = node.next_i64_req()? as usize;
							let capacity = node.next_i64_req()? as usize;
							level_map.insert(level, capacity);
						}
						Some(level_map)
					}
				};

				let all_slots = node.query_all_t::<Slots>("scope() > slots")?;
				let mut standard_slots = None;
				let mut bonus_slots = Vec::with_capacity(1);
				for slots in all_slots {
					if matches!(slots, Slots::Standard { .. }) {
						standard_slots = Some(slots);
					} else {
						bonus_slots.push(slots);
					}
				}

				let spell_capacity = {
					let mut node = node.query_req("scope() > kind")?;
					match node.next_str_req()? {
						"Prepared" => {
							let capacity = node.query_req_t("scope() > capacity")?;
							spellcasting::Capacity::Prepared(capacity)
						}
						"Known" => {
							let capacity = {
								let node = node.query_req("scope() > capacity")?;
								let mut capacity = BTreeMap::new();
								for node in &mut node.query_all("scope() > level")? {
									let level = node.next_i64_req()? as usize;
									let amount = node.next_i64_req()? as usize;
									capacity.insert(level, amount);
								}
								capacity
							};
							spellcasting::Capacity::Known(capacity)
						}
						name => {
							return Err(NotInList(name.into(), vec!["Known", "Prepared"]).into());
						}
					}
				};

				let spell_entry = SpellEntry {
					ability: ability,
					source: std::path::PathBuf::from(&class_name),
					classified_as: Some(class_name.clone()),
					cast_via_slot: true,
					cast_via_ritual: true,
					cast_via_uses: None,
					range: None,
					forced_rank: None,
				};

				let ritual_capability = match node.query_opt("scope() > ritual")? {
					None => None,
					Some(node) => {
						let available_spells =
							node.query_opt("scope() > available-spells")?.is_some();
						let selected_spells =
							node.query_opt("scope() > selected-spells")?.is_some();
						Some(RitualCapability {
							available_spells,
							selected_spells,
						})
					}
				};

				Operation::Caster(Caster {
					class_name,
					ability,
					restriction,
					cantrip_capacity,
					standard_slots,
					bonus_slots,
					spell_capacity,
					spell_entry,
					ritual_capability,
				})
			}
			Some("add_source") => {
				let class_name = node.get_str_req("class")?.to_owned();
				let mut spell_ids = Vec::new();
				for s in node.query_str_all("scope() > spell", 0)? {
					spell_ids.push(SourceId::from_str(s)?.with_basis(node.id(), false));
				}
				Operation::AddSource {
					class_name,
					spell_ids,
				}
			}
			Some("add_prepared") => {
				let ability = node.get_str_req_t::<Ability>("ability")?;
				let classified_as = node.get_str_opt("classified_as")?.map(str::to_owned);

				let mut specific_spells = Vec::new();
				for mut node in &mut node.query_all("scope() > spell")? {
					let id = node.next_str_req()?;
					let id = SourceId::from_str(id)?.with_basis(node.id(), false);
					let info = PreparedInfo::from_kdl(&mut node)?;
					specific_spells.push((id, info));
				}

				let selectable_spells = match node.query_opt("scope() > options")? {
					None => None,
					Some(mut node) => {
						let info = PreparedInfo::from_kdl(&mut node)?;
						let mut filter = None;
						let amount = node
							.query_opt_t("scope() > amount")?
							.unwrap_or(Value::Fixed(1));
						let mut selector = selector::Object {
							id: Default::default(),
							object_category: Spell::id().into(),
							amount,
							criteria: None,
						};
						if let Some(node) = node.query_opt("scope() > filter")? {
							let spell_filter = {
								let ranks = node.query_i64_all("scope() > rank", 0)?;
								let ranks =
									ranks.into_iter().map(|v| v as u8).collect::<HashSet<_>>();
								let tags = node.query_str_all("scope() > tag", 0)?;
								let tags =
									tags.into_iter().map(str::to_owned).collect::<HashSet<_>>();
								spellcasting::Filter {
									ranks,
									tags,
									..Default::default()
								}
							};
							selector.set_criteria(spell_filter.as_criteria());
							filter = Some(spell_filter);
						}
						Some(SelectableSpells {
							filter,
							selector,
							prepared: info,
						})
					}
				};

				let limited_uses = node.query_opt_t::<LimitedUses>("scope() > limited_uses")?;
				Operation::AddPrepared {
					ability,
					classified_as,
					specific_spells,
					selectable_spells,
					limited_uses,
				}
			}
			Some(name) => {
				return Err(NotInList(name.into(), vec!["add_source", "add_prepared"]).into())
			}
		};
		Ok(Self(operation))
	}
}
// TODO AsKdl: tests for Spellcasting
impl AsKdl for Spellcasting {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match &self.0 {
			Operation::Caster(caster) => {
				node.push_entry(("ability", caster.ability.long_name()));
				node.push_entry(("class", caster.class_name.clone()));
				node.push_child({
					let mut node = NodeBuilder::default();
					for tag in &caster.restriction.tags {
						node.push_child_t("tag", tag);
					}
					node.build("restriction")
				});
				if let Some(ritual_cap) = &caster.ritual_capability {
					node.push_child({
						let mut node = NodeBuilder::default();
						if ritual_cap.available_spells {
							node.push_child(NodeBuilder::default().build("available-spells"));
						}
						if ritual_cap.selected_spells {
							node.push_child(NodeBuilder::default().build("selected-spells"));
						}
						node.build("ritual")
					});
				}
				if let Some(level_map) = &caster.cantrip_capacity {
					node.push_child_opt({
						let mut node = NodeBuilder::default();
						for (level, amt) in level_map {
							node.push_child(
								NodeBuilder::default()
									.with_entry(*level as i64)
									.with_entry(*amt as i64)
									.build("level"),
							);
						}
						node.build("cantrips")
					});
				}
				node.push_child({
					let mut node = NodeBuilder::default();
					match &caster.spell_capacity {
						spellcasting::Capacity::Prepared(eval) => {
							node.push_entry("Prepared");
							node.push_child({
								let mut node = NodeBuilder::default();
								node.append_typed("Evaluator", eval.as_kdl());
								node.build("capacity")
							});
						}
						spellcasting::Capacity::Known(level_map) => {
							node.push_entry("Known");
							node.push_child_opt({
								let mut node = NodeBuilder::default();
								for (level, amt) in level_map {
									node.push_child(
										NodeBuilder::default()
											.with_entry(*level as i64)
											.with_entry(*amt as i64)
											.build("level"),
									);
								}
								node.build("capacity")
							});
						}
					}
					node.build("kind")
				});
				if let Some(slots) = &caster.standard_slots {
					node.push_child_t("slots", slots);
				}
				for bonus_slots in &caster.bonus_slots {
					node.push_child_t("slots", bonus_slots);
				}
				node
			}
			Operation::AddSource {
				class_name,
				spell_ids,
			} => {
				node.push_entry("add_source");
				node.push_entry(("class", class_name.clone()));
				for spell_id in spell_ids {
					node.push_child_t("spell", spell_id);
				}
				node
			}
			Operation::AddPrepared {
				ability,
				classified_as,
				specific_spells,
				selectable_spells,
				limited_uses,
			} => {
				node.push_entry(("ability", ability.long_name()));
				node.push_entry("add_prepared");
				if let Some(class_name) = classified_as {
					node.push_entry(("classified_as", class_name.clone()));
				}

				for (spell_id, prepared_info) in specific_spells {
					node.push_child({
						let mut node = NodeBuilder::default();
						let spell_id = spell_id.as_kdl();
						if !spell_id.is_empty() {
							node += spell_id;
						}
						node += prepared_info.as_kdl();
						node.build("spell")
					});
				}

				if let Some(selectable) = selectable_spells {
					node.push_child({
						let mut node = NodeBuilder::default();
						node += selectable.prepared.as_kdl();
						node.push_child_t("amount", &selectable.selector.amount);
						if let Some(filter) = &selectable.filter {
							node.push_child({
								let mut node = NodeBuilder::default();
								for rank in filter.ranks.iter().sorted() {
									node.push_child_t("rank", rank);
								}
								for tag in filter.tags.iter().sorted() {
									node.push_child_t("tag", tag);
								}
								node.build("filter")
							});
						}
						node.build("options")
					});
				}

				if let Some(limited_uses) = limited_uses {
					node.push_child_t("limited_uses", limited_uses);
				}

				node
			}
		}
	}
}

impl FromKDL for PreparedInfo {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let can_cast_through_slot = node.get_bool_opt("use_slot")?.unwrap_or_default();
		let can_ritual_cast = node.get_bool_opt("use_ritual")?.unwrap_or_default();
		let cast_at_rank = node.get_i64_opt("rank")?.map(|v| v as u8);
		let range = node.query_opt_t::<spell::Range>("scope() > range")?;
		Ok(PreparedInfo {
			can_cast_through_slot,
			can_ritual_cast,
			range,
			cast_at_rank,
		})
	}
}
// TODO AsKdl: tests for PreparedInfo
impl AsKdl for PreparedInfo {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if self.can_cast_through_slot {
			node.push_entry(("use_slot", true));
		}
		if self.can_ritual_cast {
			node.push_entry(("use_ritual", true));
		}
		if let Some(rank) = &self.cast_at_rank {
			node.push_entry(("rank", *rank as i64));
		}
		if let Some(range) = &self.range {
			node.push_child_t("range", range);
		}
		node
	}
}
