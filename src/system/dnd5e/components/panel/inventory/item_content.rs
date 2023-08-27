use crate::{
	components::{database::{UseQueryAllHandle, use_query_typed, QueryStatus}, modal, progress_bar},
	database::app::Criteria,
	page::characters::sheet::{joined::editor::{description, mutator_list}, MutatorImpact},
	page::characters::sheet::CharacterHandle,
	system::{dnd5e::{
		components::{
			panel::{AvailableSpellList, HeaderAddon, spell_name_and_icons, spell_overview_info},
			validate_uint_only, FormulaInline, GeneralProp, WalletInline,
		},
		data::{
			action::ActivationKind,
			character::{spellcasting::{SpellEntry, AbilityOrStat}, MAX_SPELL_RANK, Persistent},
			item::{self, Item, container::spell::ContainerSpell},
			ArmorExtended, Spell, WeaponProficiency, Indirect,
		},
		evaluator::IsProficientWith,
	}, core::SourceId},
	utility::{Evaluator, InputExt},
};
use any_range::AnyRange;
use std::{path::PathBuf, collections::HashSet};
use yew::prelude::*;

pub fn get_inventory_item<'c>(
	state: &'c CharacterHandle,
	id_path: &Vec<uuid::Uuid>,
) -> Option<&'c Item> {
	let mut iter = id_path.iter();
	let mut item = None;
	while let Some(id) = iter.next() {
		item = match item {
			None => state.inventory().get_item(id),
			Some(prev_item) => match &prev_item.items {
				None => {
					return None;
				}
				Some(container) => container.get_item(id),
			},
		};
	}
	item
}
pub fn get_inventory_item_mut<'c>(
	persistent: &'c mut Persistent,
	id_path: &Vec<uuid::Uuid>,
) -> Option<&'c mut Item> {
	let mut iter = id_path.iter();
	let Some(id) = iter.next() else { return None; };
	let mut item = persistent.inventory.get_mut(id);
	while let Some(id) = iter.next() {
		let Some(prev_item) = item.take() else { return None; };
		let Some(container) = &mut prev_item.items else { return None; };
		item = container.get_mut(id);
	}
	item
}

#[derive(Clone, PartialEq)]
pub enum ItemLocation {
	Database {
		query: UseQueryAllHandle<Item>,
		index: usize,
	},
	Inventory {
		id_path: Vec<uuid::Uuid>,
	},
}
impl ItemLocation {
	pub fn resolve<'c>(&'c self, state: &'c CharacterHandle) -> Option<&'c Item> {
		match self {
			Self::Database { query, index } => query.get(*index),
			Self::Inventory { id_path } => get_inventory_item(state, id_path),
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct ItemBodyProps {
	pub location: ItemLocation,
	#[prop_or_default]
	pub on_quantity_changed: Option<Callback<u32>>,
	#[prop_or_default]
	pub is_equipped: bool,
	#[prop_or_default]
	pub set_equipped: Option<Callback<bool>>,
}
#[function_component]
pub fn ItemInfo(props: &ItemBodyProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let Some(item) = props.location.resolve(&state) else { return Html::default(); };

	let mut sections = Vec::new();
	if IsProficientWith::Tool(item.name.clone()).evaluate(&state) {
		sections.push(html! {
			<div class="property">
				<strong>{"Proficient (with tool):"}</strong>
				<span>{"✔ Yes"}</span>
			</div>
		});
	}
	if let Some(rarity) = item.rarity {
		sections.push(html! {
			<div class="property">
				<strong>{"Rarity:"}</strong>
				<span>{rarity}</span>
			</div>
		});
	}
	if !item.worth.is_empty() {
		sections.push(html! {
			<div class="property">
				<strong>{"Worth:"}</strong>
				<span><WalletInline wallet={item.worth} /></span>
			</div>
		});
	}
	if item.weight > 0.0 {
		sections.push(html! {
			<div class="property">
				<strong>{"Weight:"}</strong>
				<span>{item.weight * item.quantity() as f32}{" lb."}</span>
			</div>
		});
	}
	match &item.kind {
		item::Kind::Simple { count } => {
			let inner = match (&props.on_quantity_changed, item.can_stack()) {
				(None, _) | (Some(_), false) => html! { <span>{count}</span> },
				(Some(on_changed), true) => {
					let count = *count;
					let increment = Callback::from({
						let on_changed = on_changed.clone();
						move |_| {
							on_changed.emit(count.saturating_add(1));
						}
					});
					let decrement = Callback::from({
						let on_changed = on_changed.clone();
						move |_| {
							on_changed.emit(count.saturating_sub(1));
						}
					});
					let onchange = Callback::from({
						let on_changed = on_changed.clone();
						move |evt: web_sys::Event| {
							let Some(value) = evt.input_value_t::<u32>() else { return; };
							on_changed.emit(value);
						}
					});
					html! {
						<div class="input-group item-quantity-inline">
							<button type="button" class="btn btn-theme" onclick={decrement}><i class="bi bi-dash" /></button>
							<input
								class="form-control text-center"
								type="number"
								min="0" value={count.to_string()}
								onkeydown={validate_uint_only()}
								onchange={onchange}
							/>
							<button type="button" class="btn btn-theme" onclick={increment}><i class="bi bi-plus" /></button>
						</div>
					}
				}
			};
			sections.push(html! {
				<div class="property">
					<strong>{"Quantity:"}</strong>
					{inner}
				</div>
			});
		}
		item::Kind::Equipment(equipment) => {
			let mut equip_sections = Vec::new();
			if let Some(on_equipped) = props.set_equipped.clone() {
				let onchange = Callback::from({
					move |evt: web_sys::Event| {
						let Some(checked) = evt.input_checked() else { return; };
						on_equipped.emit(checked);
					}
				});
				equip_sections.push(html! {
					<div class="form-check">
						<input  id="equipItem" class="form-check-input equip" type="checkbox" checked={props.is_equipped} {onchange} />
						<label for="equipItem" class="form-check-label">
							{match props.is_equipped {
								true => format!("Equipped"),
								false => format!("Not Equipped"),
							}}
						</label>
					</div>
				});
			}
			if !equipment.mutators.is_empty() {
				let mut criteria_html = None;
				if let Some(criteria) = &equipment.criteria {
					criteria_html = Some(html! {
						<div>
							<div>{"Only if:"}</div>
							<span>{criteria.description().unwrap_or_else(|| format!("criteria missing description"))}</span>
						</div>
					});
				}
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<div>{"You gain the following benefits while this item is equipped:"}</div>
						{mutator_list(&equipment.mutators, None::<&CharacterHandle>)}
						{criteria_html.unwrap_or_default()}
					</div>
				});
			}
			if let Some(shield_bonus) = &equipment.shield {
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Shield"}</strong>
						<div class="ms-3">
							<div class="property">
								<strong>{"Proficient:"}</strong>
								{match IsProficientWith::Armor(ArmorExtended::Shield).evaluate(&state) {
									true => html! { <span>{"✔ Yes"}</span> },
									false => html! { <span>{"❌ No"}</span> },
								}}
							</div>
							<div class="property">
								<strong>{"Armor Class Bonus:"}</strong>
								<span>{format!("{shield_bonus:+}")}</span>
							</div>
						</div>
					</div>
				});
			}
			if let Some(armor) = &equipment.armor {
				let mut armor_sections = Vec::new();
				armor_sections.push(html! {
					<div class="property">
						<strong>{"Type:"}</strong>
						<span>{armor.kind.to_string()}</span>
					</div>
				});
				armor_sections.push(html! {
					<div class="property">
						<strong>{"Proficient:"}</strong>
						{match IsProficientWith::Armor(ArmorExtended::Kind(armor.kind)).evaluate(&state) {
							true => html! { <span>{"✔ Yes"}</span> },
							false => html! { <span>{"❌ No"}</span> },
						}}
					</div>
				});
				armor_sections.push(html! {
					<div class="property">
						<strong>{"Armor Class Formula:"}</strong>
						<span><FormulaInline formula={armor.formula.clone()} /></span>
					</div>
				});
				if let Some(min_score) = &armor.min_strength_score {
					armor_sections.push(html! {
						<div class="property">
							<strong>{"Minimum Strength Score:"}</strong>
							<span>{min_score}</span>
						</div>
					});
				}
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Armor"}</strong>
						<div class="ms-3">
							{armor_sections}
						</div>
					</div>
				});
			}
			if let Some(weapon) = &equipment.weapon {
				let mut weapon_sections = Vec::new();
				weapon_sections.push(html! {
					<div class="property">
						<strong>{"Type:"}</strong>
						<span>{weapon.kind}</span>
					</div>
				});
				weapon_sections.push(html! {
					<div class="property">
						<strong>{"Classification:"}</strong>
						<span>{weapon.classification.clone()}</span>
					</div>
				});
				let is_proficient = vec![
					IsProficientWith::Weapon(WeaponProficiency::Kind(weapon.kind)),
					IsProficientWith::Weapon(WeaponProficiency::Classification(
						weapon.classification.clone(),
					)),
				];
				let is_proficient = is_proficient.into_iter().any(|eval| eval.evaluate(&state));
				weapon_sections.push(html! {
					<div class="property">
						<strong>{"Proficient:"}</strong>
						{match is_proficient {
							true => html! { <span>{"✔ Yes"}</span> },
							false => html! { <span>{"❌ No"}</span> },
						}}
					</div>
				});
				if let Some(reach) = weapon.melee_reach() {
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Melee Attack Reach:"}</strong>
							<span>{reach}{" ft."}</span>
						</div>
					});
				}
				if let Some((short, long)) = weapon.range() {
					// TODO: find a way to communicate attack range better:
					// - normal if the target is at or closer than `short`
					// - made a disadvantage when the target is father than `short`, but closer than `long`
					// - impossible beyond the `long` range
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Range:"}</strong>
							<span>{format!("{short} ft. / {long} ft.")}</span>
						</div>
					});
				}
				if let Some(damage) = &weapon.damage {
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Damage:"}</strong>
							<span>
								{match (&damage.roll, damage.bonus) {
									(None, bonus) => bonus.to_string(),
									(Some(roll), 0) => roll.to_string(),
									(Some(roll), bonus) => format!("{}{bonus:+}", roll.to_string()),
								}}
								<span style="margin-left: 0.5rem;">{damage.damage_type.display_name()}</span>
							</span>
						</div>
					});
				}
				if !weapon.properties.is_empty() {
					weapon_sections.push(html! {
						<div class="property">
							<strong>{"Properties:"}</strong>
							<ul>
								{weapon.properties.iter().map(|property| html! {
									<li>
										<div class="property">
											<strong>{property.display_name()}{":"}</strong>
											<span>{property.description()}</span>
										</div>
									</li>
								}).collect::<Vec<_>>()}
							</ul>
						</div>
					});
				}
				equip_sections.push(html! {
					<div class="border-bottom-theme-muted">
						<strong>{"Weapon"}</strong>
						<div class="ms-3">
							{weapon_sections}
						</div>
					</div>
				});
			}
			if let Some(_attunement) = &equipment.attunement {
				// TODO: Display attunement
				// (if mutable) (un)attune button: disabled when all slots filled and not currently attuned
				// mutators & criteria applied when attuned
				// warning if attuned and not currently equipped
			}
			sections.push(html! {
				<div>
					<strong>{"Equipment"}</strong>
					<div class="ms-3">
						{equip_sections}
					</div>
				</div>
			});
		}
	}

	if let Some(spell_container) = &item.spells {
		let mut container_sections = Vec::new();

		container_sections.push(html! {
			<div class="property">
				<strong>{"Transcribe Spells From:"}</strong>
				<span>
					{match spell_container.can_transcribe_from {
						true => html!("✔"),
						false => html!("❌"),
					}}
				</span>
			</div>
		});
		container_sections.push(html! {
			<div class="property">
				<strong>{"Prepare Contained Spells:"}</strong>
				<span>
					{match spell_container.can_prepare_from {
						true => html!("✔"),
						false => html!("❌"),
					}}
				</span>
			</div>
		});
		container_sections.push(html! {
			<div class="property">
				<strong>{"Casting: "}</strong>
				{match &spell_container.casting {
					None => html!("❌"),
					Some(casting) => html! {
						<div class="ms-3">
							{match &casting.activation_kind {
								None => html!(),
								Some(activation) => html! {
									<div class="property">
										<strong>{"Casting Time:"}</strong>
										<span>
											{match activation {
												ActivationKind::Action => html!("Action"),
												ActivationKind::Bonus => html!("Bonus Action"),
												ActivationKind::Reaction => html!("Reaction"),
												ActivationKind::Special => html!("Special"),
												ActivationKind::Minute(amt) => html!(format!("{amt} Minutes")),
												ActivationKind::Hour(amt) => html!(format!("{amt} Hours")),
											}}
										</span>
									</div>
								},
							}}
							<div class="property">
								<strong>{"Destroy Item on All Consumed:"}</strong>
								<span>
									{match casting.consume_item {
										true => html!("Destroyed"),
										false => html!("Not Destroyed"),
									}}
								</span>
							</div>
							<div class="property">
								<strong>{"Consume Spell on Use:"}</strong>
								<span>
									{match casting.consume_spell {
										true => html!("Consumed"),
										false => html!("Not Consumed"),
									}}
								</span>
							</div>
							{match casting.save_dc {
								None => html!(),
								Some(dc) => html! {
									<div class="property">
										<strong>{"Spell Save DC:"}</strong>
										<span>{dc}</span>
									</div>
								},
							}}
							{match casting.attack_bonus {
								None => html!(),
								Some(atk_bonus) => html! {
									<div class="property">
										<strong>{"Spell Attack:"}</strong>
										<span>{format!("{atk_bonus:+}")}</span>
									</div>
								},
							}}
						</div>
					},
				}}
			</div>
		});

		let mut capacity_sections = Vec::new();
		if let Some(max_count) = &spell_container.capacity.max_count {
			capacity_sections.push(html! {
				<div class="property">
					<strong>{"Total Stored Spells"}</strong>
					<progress_bar::Ticked
						classes={"mt-4"}
						ticked_bar_range={progress_bar::TickDisplay::BoundsOnly}
						bar_range={0..=(*max_count as i64)}
						value_range={AnyRange::from(..=spell_container.spells.len() as i64)}
						show_labels={true}
					/>
				</div>
			});
		}
		if let Some(rank_total) = &spell_container.capacity.rank_total {
			// TODO: Query the contained spell ids so we have them on hand
			let current_value: i64 = spell_container
				.spells
				.iter()
				.map(|contained| {
					match &contained.rank {
						Some(rank) => *rank as i64,
						None => 0 as i64, // TODO: get rank for that spell id
					}
				})
				.sum();
			capacity_sections.push(html! {
				<div class="property">
					<strong>{"Total Stored Ranks"}</strong>
					<progress_bar::Ticked
						classes={"mt-4"}
						ticked_bar_range={progress_bar::TickDisplay::BoundsOnly}
						bar_range={0..=(*rank_total as i64)}
						value_range={AnyRange::from(..=current_value)}
						show_labels={true}
					/>
				</div>
			});
		}
		let rank_min = spell_container.capacity.rank_min.unwrap_or(0);
		let rank_max = spell_container.capacity.rank_max.unwrap_or(MAX_SPELL_RANK);
		if rank_min != 0 || rank_max != MAX_SPELL_RANK {
			capacity_sections.push({
				html! {
					<div class="property">
						<strong>{"Allowed Spell Ranks"}</strong>
						<progress_bar::Ticked
							classes={"mt-4"}
							ticked_bar_range={progress_bar::TickDisplay::AllTicks}
							bar_range={0..=(MAX_SPELL_RANK as i64)}
							value_range={AnyRange::from((rank_min as i64)..=(rank_max as i64))}
							show_labels={true}
						/>
					</div>
				}
			});
		}

		if !capacity_sections.is_empty() {
			container_sections.push(html! {
				<div>
					<strong>{"Storage Capacity"}</strong>
					<div class="ms-3 capacity">
						{capacity_sections}
					</div>
				</div>
			});
		}

		let browse = match &props.location {
			ItemLocation::Database { .. } => html!(),
			ItemLocation::Inventory { id_path } => {
				let onclick = modal_dispatcher.callback({
					let id_path = id_path.clone();
					move |_| {
						modal::Action::Open(modal::Props {
							centered: true,
							scrollable: true,
							root_classes: classes!("browse", "item-spell-container"),
							content: html! {<ModalSpellContainerBrowser value={id_path.clone()} />},
							..Default::default()
						})
					}
				});
				html!(
					<button class="btn btn-outline-theme btn-xs" type="button" {onclick}>
						{"Browse Spells"}
					</button>
				)
			}
		};
		let contents = html! {
			<div>
				{"TODO Show spell contents"}
			</div>
		};

		sections.push(html! {
			<div>
				<strong>{"Spell Container"}</strong>
				<div class="ms-3 spell-container">
					{container_sections}
					{browse}
					{contents}
				</div>
			</div>
		});
	}
	if !item.description.is_empty() {
		let desc = item.description.clone().evaluate(&state);
		sections.push(description(&desc, false, false));
	}
	if let Some(notes) = &item.notes {
		sections.push(html! {
			<div class="property">
				<strong>{"Notes."}</strong>
				<span class="text-block">{notes.clone()}</span>
			</div>
		});
	}
	if !item.tags.is_empty() {
		sections.push(html! {
			<div class="property">
				<strong>{"Tags:"}</strong>
				<span>{item.tags.join(", ")}</span>
			</div>
		});
	}
	html! {<>
		{sections}
	</>}
}

#[function_component]
fn ModalSpellContainerBrowser(GeneralProp { value }: &GeneralProp<Vec<uuid::Uuid>>) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();

	let fetch_indirect_spells = use_query_typed::<Spell>();
	let indirect_spell_ids = use_state_eq(|| Vec::new());
	use_effect_with_deps(
		{
			let fetch_indirect_spells = fetch_indirect_spells.clone();
			move |ids: &UseStateHandle<Vec<SourceId>>| {
				fetch_indirect_spells.run((**ids).clone());
			}
		},
		indirect_spell_ids.clone(),
	);

	let Some(item) = get_inventory_item(&state, value) else { return Html::default(); };
	let Some(spell_container) = &item.spells else { return Html::default(); };

	indirect_spell_ids.set(spell_container.spells.iter().filter_map(|contained| {
		match &contained.spell {
			Indirect::Id(id) => Some(id.unversioned()),
			Indirect::Custom(_spell) => None,
		}
	}).collect::<Vec<_>>());
	let contained_ids = spell_container.spells.iter().map(|contained| {
		match &contained.spell {
			Indirect::Id(id) => id,
			Indirect::Custom(spell) => &spell.id,
		}.unversioned()
	}).collect::<HashSet<_>>();

	let add_to_container = state.new_dispatch({
		let id_path = value.clone();
		move |spell: Indirect<Spell>, persistent: &mut Persistent| {
			let Some(item) = get_inventory_item_mut(persistent, &id_path) else {
				return MutatorImpact::None;
			};
			let Some(spell_container) = &mut item.spells else {
				return MutatorImpact::None;
			};
			spell_container.spells.push(ContainerSpell {
				spell,
				rank: None,
				save_dc: None,
				attack_bonus: None,
			});
			// TODO: only recompile character when the modal is dismissed
			return MutatorImpact::None;
		}
	});

	// TODO: implement SpellEntry for spell container when adding spells to the spell panel. For now lets just use /something/.
	let (casting_atk_bonus, casting_dc) = match &spell_container.casting {
		None => (None, None),
		Some(casting) => (casting.attack_bonus, casting.save_dc),
	};

	let rank_min = spell_container.capacity.rank_min.unwrap_or(0);
	let rank_max = spell_container.capacity.rank_max.unwrap_or(MAX_SPELL_RANK);
	let criteria = Criteria::contains_prop(
		"rank",
		Criteria::any((rank_min..=rank_max).into_iter().map(|rank| Criteria::exact(rank))),
	);
	
	let contained_spells = spell_container.spells.iter().map(|contained| {
		let ContainerSpell {
			spell,
			rank,
			save_dc,
			attack_bonus,
		} = contained;
		let spell = match spell {
			Indirect::Id(id) => match fetch_indirect_spells.status() {
				QueryStatus::Success((_ids, spells_by_id)) => {
					spells_by_id.get(id)
				}
				_ => None,
			},
			Indirect::Custom(spell) => Some(spell),
		};
		let Some(spell) = spell else { return Html::default(); };

		let entry = SpellEntry {
			source: PathBuf::new(),
			classified_as: None,
			cast_via_slot: false,
			cast_via_ritual: false,
			cast_via_uses: None,
			range: None,
			rank: *rank,
			attack_bonus: AbilityOrStat::Stat(attack_bonus.or(casting_atk_bonus).unwrap_or(0)),
			save_dc: AbilityOrStat::Stat(save_dc.or(casting_dc).unwrap_or(0)),
			// TODO: Should this also be an abilityorstat for the caster's original ability modifier?
			damage_ability: None,
		};

		// TODO: Next up: fields for rank, save_dc, and attack_bonus on the container spell (so users can adjust the values when a spell is added).
		// Save DC & Attack Bonus are not available if the item has casting values for those already set,
		// but are required if those casting values are not provided.

		html! {
			<div class="spell-row">
				<div class="name-and-source">
					{spell_name_and_icons(&state, spell, &entry, false)}
				</div>
				{spell_overview_info(&state, spell, &entry, None)}
			</div>
		}
	}).collect::<Vec<_>>();

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Spell Container Browser"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div class="spell-section">
				{contained_spells}
			</div>
			<AvailableSpellList
				header_addon={HeaderAddon::from({
					move |spell: &Spell| -> Html {
						let contained = contained_ids.contains(&spell.id.unversioned());
						let mut classes = classes!("btn", "btn-xs", "select");
						if contained {
							classes.push("btn-outline-secondary");
						}
						else {
							classes.push("btn-outline-theme");
						}
						html! {
							<button
								role="button"
								class={classes}
								disabled={contained}
								onclick={add_to_container.reform({
									let spell = Indirect::Id(spell.id.unversioned());
									move |_| spell.clone()
								})}
							>
								{match contained {
									true => "Added",
									false => "Add",
								}}
							</button>
						}
					}
				})}
				criteria={criteria.clone()}
				entry={None}
			/>
		</div>
	</>}
}
