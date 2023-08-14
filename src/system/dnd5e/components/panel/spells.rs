use crate::{
	components::{database::use_typed_fetch_callback, modal, stop_propagation, Spinner},
	database::app::{Database, QueryDeserialize},
	page::characters::sheet::joined::editor::{CollapsableCard, DescriptionSection},
	page::characters::sheet::CharacterHandle,
	page::characters::sheet::MutatorImpact,
	system::{
		self,
		core::{ModuleId, SourceId},
		dnd5e::{
			components::glyph::Glyph,
			data::{
				character::{
					spellcasting::{CasterKind, RitualCapability, SpellEntry},
					MAX_SPELL_RANK,
				},
				proficiency, spell, Spell,
			},
			DnD5e,
		},
	},
	utility::InputExt,
};
use convert_case::{Case, Casing};
use futures_util::{FutureExt, StreamExt};
use itertools::Itertools;
use std::{collections::BTreeMap, pin::Pin};
use yew::prelude::*;

fn rank_suffix(rank: u8) -> &'static str {
	match rank {
		1 => "st",
		2 => "nd",
		3 => "rd",
		4..=9 => "th",
		_ => "",
	}
}

#[function_component]
pub fn Spells() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();

	let mut sections = SpellSections::default();
	sections.insert_slots(&state);
	sections.insert_selected_spells(&state);
	sections.insert_permaprepared_spells(&state);
	sections.insert_available_ritual_spells(&state);

	let sections = {
		let mut html = Vec::new();
		for (rank, section_props) in sections.sections {
			if section_props.spells.is_empty() && (section_props.slot_count.is_none() || rank == 0)
			{
				continue;
			}
			html.push(spell_section(&state, rank, section_props));
		}
		html! {<>{html}</>}
	};

	let feature_stats = state.spellcasting().has_casters().then(|| {
		use unzip_n::unzip_n;
		unzip_n!(4);
		let (names, modifier, atk_bonus, save_dc) = state
			.spellcasting()
			.iter_casters()
			.map(|caster| {
				let name = caster.name().clone();
				let modifier = state.ability_modifier(caster.ability, None);
				let atk_bonus =
					state.ability_modifier(caster.ability, Some(proficiency::Level::Full));
				let save_dc = 8 + atk_bonus;
				(name, modifier, atk_bonus, save_dc)
			})
			.unzip_n_vec();
		let names = names.into_iter().sorted().map(|caster_id| {
			html! {
				<ManageCasterButton {caster_id} />
			}
		});
		let names = Itertools::intersperse(names, html! { <span class="mx-1">{"|"}</span> })
			.collect::<Vec<_>>();
		let modifier = modifier.into_iter().map(|v| format!("{v:+}")).join(" | ");
		let atk_bonus = atk_bonus.into_iter().map(|v| format!("{v:+}")).join(" | ");
		let save_dc = save_dc.into_iter().map(|v| format!("{v}")).join(" | ");
		html! {
			<div class="caster-stats">
				<div class="names">{names}</div>
				<div class="row">
					<div class="col mod">
						<div class="stats">{modifier}</div>
						<div class="title">{"Modifier"}</div>
					</div>
					<div class="col atk">
						<div class="stats">{atk_bonus}</div>
						<div class="title">{"Spell Attack"}</div>
					</div>
					<div class="col dc">
						<div class="stats">{save_dc}</div>
						<div class="title">{"Save DC"}</div>
					</div>
				</div>
			</div>
		}
	});

	html! {
		<div class="panel spells">
			{feature_stats.unwrap_or_default()}

			<div class="input-group search my-2">
				<span class="input-group-text"><i class="bi bi-search"/></span>
				<input
					type="text" class="form-control"
					placeholder="Search spell names, tags, or other properties"
				/>
			</div>

			<div class="sections">
				{sections}
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct CasterNameProps {
	caster_id: AttrValue,
}
#[function_component]
fn ManageCasterButton(CasterNameProps { caster_id }: &CasterNameProps) -> Html {
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_browser = modal_dispatcher.callback({
		let caster_id = caster_id.clone();
		move |_| {
			let caster_id = caster_id.clone();
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("spells", "browse"),
				content: html! {<ManagerCasterModal {caster_id} />},
				..Default::default()
			})
		}
	});
	html! {
		<button type="button" class="btn btn-xs btn-outline-theme mb-1" onclick={open_browser}>
			{caster_id.clone()}
		</button>
	}
}

struct SpellSections<'c> {
	sections: BTreeMap<u8, SectionProps<'c>>,
}
impl<'c> Default for SpellSections<'c> {
	fn default() -> Self {
		Self {
			sections: (0..=MAX_SPELL_RANK)
				.map(|slot: u8| (slot, SectionProps::default()))
				.collect::<BTreeMap<_, _>>(),
		}
	}
}
impl<'c> SpellSections<'c> {
	fn insert_slots(&mut self, state: &'c CharacterHandle) {
		if let Some(slots) = state.spellcasting().spell_slots(&*state) {
			for (rank, slot_count) in slots {
				let section =
					self.sections.get_mut(&rank).expect(&format!(
					"Spell rank {rank} is not supported by UI, must be in the range of [0, {}].", MAX_SPELL_RANK
				));
				let data_path = state.persistent().selected_spells.consumed_slots_path(rank);
				let consumed_slots = state.get_first_selection_at::<usize>(&data_path);
				let consumed_slots = consumed_slots.map(Result::ok).flatten().unwrap_or_default();
				section.slot_count = Some((consumed_slots, slot_count));
			}
		}
	}

	fn insert_spell(&mut self, spell: &'c Spell, entry: &'c SpellEntry, location: SpellLocation) {
		let ranks = match (entry.forced_rank, entry.cast_via_slot) {
			(Some(rank), _) => vec![(rank, location)],
			(None, false) => vec![(spell.rank, location)],
			(None, true) => {
				let max_rank = match spell.rank {
					0 => 0,
					_ => MAX_SPELL_RANK,
				};
				let rank_range = spell.rank..=max_rank;
				let mut locations = Vec::with_capacity(rank_range.len());
				locations.resize(rank_range.len(), location);
				rank_range.zip(locations).collect::<Vec<_>>()
			}
		};
		for (section_rank, location) in ranks {
			let Some(section) = self.sections.get_mut(&section_rank) else { continue; };
			if section_rank == 0 || section.slot_count.is_some() {
				section.insert_spell(spell, entry, location);
			}
		}
	}

	fn insert_selected_spells(&mut self, state: &'c CharacterHandle) {
		for caster_id in state.persistent().selected_spells.iter_caster_ids() {
			let Some(caster) = state.spellcasting().get_caster(caster_id) else { continue; };
			let Some(iter_spells) = state.persistent().selected_spells.iter_caster(caster_id) else { continue; };
			for spell in iter_spells {
				self.insert_spell(
					spell,
					&caster.spell_entry,
					SpellLocation::Selected {
						caster_id: caster_id.clone(),
						spell_id: spell.id.unversioned(),
					},
				);
			}
		}
	}

	fn insert_permaprepared_spells(&mut self, state: &'c CharacterHandle) {
		for (_id, spell_entry) in state.spellcasting().prepared_spells() {
			let Some(spell) = &spell_entry.spell else { continue; };
			for (source, entry) in &spell_entry.entries {
				self.insert_spell(
					spell,
					entry,
					SpellLocation::AlwaysPrepared {
						spell_id: spell.id.unversioned(),
						source: source.clone(),
					},
				);
				self.insert_as_ritual(spell, entry, RitualSpellSource::AlwaysPrepared(source.clone()));
			}
		}
	}

	fn insert_available_ritual_spells(&mut self, state: &'c CharacterHandle) {
		for (caster_id, spell, spell_entry) in state.spellcasting().iter_ritual_spells() {
			self.insert_as_ritual(spell, spell_entry, RitualSpellSource::Caster(caster_id.clone()));
		}
	}

	fn insert_as_ritual(&mut self, spell: &'c Spell, spell_entry: &'c SpellEntry, source: RitualSpellSource) {
		if !spell_entry.cast_via_ritual {
			return;
		}
		// ritual-only spells can only be cast at their specified rank
		let Some(section) = self.sections.get_mut(&spell.rank) else { return; };
		section.insert_spell(
			spell,
			spell_entry,
			SpellLocation::AvailableAsRitual {
				spell_id: spell.id.unversioned(),
				source,
			},
		);
	}
}

#[derive(Default)]
struct SectionProps<'c> {
	slot_count: Option<(/*consumed*/ usize, /*max*/ usize)>,
	spells: Vec<SectionSpell<'c>>,
}
struct SectionSpell<'c> {
	spell: &'c Spell,
	entry: &'c SpellEntry,
	location: SpellLocation,
}
#[derive(Clone, PartialEq, Debug)]
enum SpellLocation {
	Selected {
		spell_id: SourceId,
		caster_id: String,
	},
	AlwaysPrepared {
		spell_id: SourceId,
		source: std::path::PathBuf,
	},
	AvailableAsRitual {
		spell_id: SourceId,
		source: RitualSpellSource,
	},
}
#[derive(Clone, PartialEq, Debug)]
enum RitualSpellSource {
	Caster(String),
	AlwaysPrepared(std::path::PathBuf),
}
impl SpellLocation {
	fn get<'this, 'c>(
		&'this self,
		state: &'c CharacterHandle,
	) -> Option<(&'c Spell, &'c SpellEntry)> {
		match self {
			SpellLocation::Selected {
				caster_id,
				spell_id,
			} => {
				let Some(caster) = state.spellcasting().get_caster(caster_id) else { return None; };
				let Some(spell) = state.persistent().selected_spells.get_spell(caster_id, spell_id) else { return None; };
				Some((spell, &caster.spell_entry))
			}
			SpellLocation::AlwaysPrepared { spell_id, source } => {
				let Some(spell_entry) = state.spellcasting().prepared_spells().get(spell_id) else { return None; };
				let Some(spell) = &spell_entry.spell else { return None; };
				let Some(entry) = spell_entry.entries.get(source) else { return None; };
				Some((spell, entry))
			}
			SpellLocation::AvailableAsRitual {
				spell_id,
				source: RitualSpellSource::Caster(caster_id),
			} => {
				let Some(caster) = state.spellcasting().get_caster(caster_id) else { return None; };
				let Some(spell) = state.spellcasting().get_ritual(spell_id) else { return None; };
				Some((spell, &caster.spell_entry))
			}
			SpellLocation::AvailableAsRitual {
				spell_id,
				source: RitualSpellSource::AlwaysPrepared(source),
			} => {
				let Some(spell_entry) = state.spellcasting().prepared_spells().get(spell_id) else { return None; };
				let Some(spell) = &spell_entry.spell else { return None; };
				let Some(entry) = spell_entry.entries.get(source) else { return None; };
				Some((spell, entry))
			}
		}
	}
}
impl<'c> SectionProps<'c> {
	pub fn insert_spell(
		&mut self,
		spell: &'c Spell,
		entry: &'c SpellEntry,
		location: SpellLocation,
	) {
		let idx = self.spells.binary_search_by(|row| {
			let spell_name = row.spell.name.cmp(&spell.name);
			let source = row.entry.source.cmp(&entry.source);
			spell_name.then(source)
		});
		let idx = idx.unwrap_or_else(|e| e);
		self.spells.insert(
			idx,
			SectionSpell {
				spell,
				entry,
				location,
			},
		);
	}
}

fn spell_section<'c>(
	state: &'c CharacterHandle,
	rank: u8,
	section_props: SectionProps<'c>,
) -> Html {
	let suffix = rank_suffix(rank);
	let rank_text = match rank {
		0 => "cantrip",
		_ => "level",
	};
	let title = match rank {
		0 => rank_text.to_case(Case::Upper),
		rank => format!(
			"{rank}{}",
			format!("{suffix} {rank_text}").to_case(Case::Upper)
		),
	};
	let inline_rank_text = format!("{rank}{suffix}-{rank_text}");
	let empty_note = format!(
		"You do not have any {inline_rank_text} spells or spells that scale to \
		{inline_rank_text} available, but you can cast lower level spells \
		using your {inline_rank_text} spell slots."
	);

	let slots = section_props.slot_count.as_ref().map(|(consumed, count)| {
		let toggle_slot = state.new_dispatch({
			let consumed_slots = *consumed;
			move |evt: web_sys::Event, persistent| {
				let Some(consume_slot) = evt.input_checked() else { return MutatorImpact::None; };
				let new_consumed_slots = match consume_slot {
					true => consumed_slots.saturating_add(1),
					false => consumed_slots.saturating_sub(1),
				};
				let data_path = persistent.selected_spells.consumed_slots_path(rank);
				persistent.set_selected_value(&data_path, new_consumed_slots.to_string());
				MutatorImpact::None
			}
		});
		html! {
			<div class="slots">
				{(0..*count).map(|idx| html! {
					<input
					type="checkbox"
						class={"form-check-input slot"}
						checked={idx < *consumed}
						onchange={toggle_slot.clone()}
					/>
				}).collect::<Vec<_>>()}
				<span class="ms-1">{"SLOTS"}</span>
			</div>
		}
	});

	let contents = match section_props.spells.is_empty() {
		true => html! { <p class="empty-note mx-4">{empty_note}</p> },
		false => {
			html! {
				<table class="table table-compact mx-auto">
					<thead>
						<tr style="font-size: 11px;">
							<th scope="col"></th>
							<th scope="col">{"Name"}</th>
							<th scope="col">{"Time"}</th>
							<th scope="col">{"Range"}</th>
							<th scope="col">{"Hit / DC"}</th>
							<th scope="col">{"Effect"}</th>
						</tr>
					</thead>
					<tbody>
						{section_props.spells.into_iter().map(|section_spell| {
							spell_row(SpellRowProps {
								state,
								section_rank: rank,
								slots: section_props.slot_count,
								section_spell,
							})
						}).collect::<Vec<_>>()}
					</tbody>
				</table>
			}
		}
	};
	html! {
		<div class="spell-section mb-2">
			<div class="header">
				<div class="title">{title}</div>
				{slots.unwrap_or_default()}
			</div>
			{contents}
		</div>
	}
}

struct SpellRowProps<'c> {
	state: &'c CharacterHandle,
	section_rank: u8,
	slots: Option<(usize, usize)>,
	section_spell: SectionSpell<'c>,
}
fn spell_row<'c>(props: SpellRowProps<'c>) -> Html {
	use spell::CastingDuration;
	let SpellRowProps {
		state,
		section_rank,
		slots,
		section_spell: SectionSpell {
			spell,
			entry,
			location,
		},
	} = props;

	let (use_kind, src_text_suffix) = match (&location, spell.rank, entry.cast_via_uses.as_ref()) {
		(SpellLocation::AvailableAsRitual { .. }, _, _) => (UseSpell::RitualOnly, None),
		(_, 0, None) => (UseSpell::AtWill, None),
		(_, spell_rank, None) => {
			let slot = UseSpell::Slot {
				spell_rank,
				slot_rank: section_rank,
				slots: slots.clone(),
			};
			(slot, None)
		}
		(_, _, Some(limited_uses)) => {
			let data_path = limited_uses.get_uses_path(state);
			let max_uses = limited_uses.get_max_uses(state) as u32;
			let uses_consumed = limited_uses.get_uses_consumed(state);
			let kind = UseSpell::LimitedUse {
				uses_consumed,
				max_uses,
				data_path,
			};
			let text = html! {
				<span class="ms-1">
					{format!(
						"({}/{max_uses}{})",
						max_uses.saturating_sub(uses_consumed),
						limited_uses.get_reset_rest(state).map(|rest| {
							format!(" per {} rest", rest.to_string())
						}).unwrap_or_default()
					)}
				</span>
			};
			(kind, Some(text))
		}
	};

	let can_ritual_cast = spell.casting_time.ritual && {
		use_kind == UseSpell::RitualOnly || {
			let classified = entry.classified_as.as_ref();
			let caster = classified
				.map(|id| state.spellcasting().get_caster(id))
				.flatten();
			let ritual_casting = caster
				.map(|caster| caster.ritual_capability.as_ref())
				.flatten();
			let ritual_cast_selected = ritual_casting
				.map(|ritual| ritual.selected_spells)
				.unwrap_or_default();
			ritual_cast_selected
		}
	};

	// TODO: tooltip for casting time duration
	// TODO: Tooltips for ritual & concentration icons
	html! {
		<SpellModalRowRoot {location}>
			<td onclick={stop_propagation()}>
				<div class="d-inline-flex align-items-center justify-content-center w-100">
					<UseSpellButton kind={use_kind} />
				</div>
			</td>
			<td>
				<div>
					{&spell.name}
					{can_ritual_cast.then(|| html!(
						<Glyph tag="div" classes={"ritual ms-1"} />
					)).unwrap_or_default()}
					{spell.duration.concentration.then(|| html!(
						<Glyph tag="div" classes={"concentration ms-1"} />
					)).unwrap_or_default()}
				</div>
				<div style="font-size: 10px; color: var(--bs-gray-600);">
					{crate::data::as_feature_path_text(&entry.source)}
					{src_text_suffix.unwrap_or_default()}
				</div>
			</td>
			<td>
				{match &spell.casting_time.duration {
					CastingDuration::Action => html!("1A"),
					CastingDuration::Bonus => html!("1BA"),
					CastingDuration::Reaction(_trigger) => html!("1R"),
					CastingDuration::Unit(amt, kind) => html!{<>{amt}{kind.chars().next().unwrap()}</>},
				}}
			</td>
			<td>
				{match entry.range.as_ref().unwrap_or(&spell.range) {
					spell::Range::OnlySelf => html!("Self"),
					spell::Range::Touch => html!("Touch"),
					spell::Range::Unit { distance, unit } => html! {<>{distance}{" "}{unit}</>},
					spell::Range::Sight => html!("Sight"),
					spell::Range::Unlimited => html!("Unlimited"),
				}}
			</td>
			<td>
				{match &spell.check {
					None => html!("--"),
					Some(spell::Check::AttackRoll(_atk_kind)) => {
						let modifier = state.ability_modifier(entry.ability, Some(proficiency::Level::Full));
						html!(format!("{modifier:+}"))
					}
					Some(spell::Check::SavingThrow(ability, fixed_dc)) => {
						let abb_name = ability.abbreviated_name().to_case(Case::Upper);
						match fixed_dc {
							Some(dc) => html!(format!("{abb_name} {dc}")),
							None => {
								let modifier = state.ability_modifier(entry.ability, Some(proficiency::Level::Full));
								let dc = 8 + modifier;
								html!(format!("{abb_name} {dc}"))
							}
						}
					}
				}}
			</td>
			<td>
				{match &spell.damage {
					None => html!("--"),
					Some(damage) => {
						let modifier = state.ability_modifier(entry.ability, Some(proficiency::Level::Full));
						let upcast_amt = section_rank - spell.rank;
						let (roll_set, bonus) = damage.evaluate(&*state, modifier, upcast_amt as u32);
						let mut spans = roll_set.rolls().into_iter().enumerate().map(|(idx, roll)| {
							html! {
								<span>
									{(idx != 0).then(|| html!("+")).unwrap_or_default()}
									{roll.to_string()}
								</span>
							}
						}).collect::<Vec<_>>();
						if bonus != 0 {
							spans.push(html! {
								<span>{format!("{bonus:+}")}</span>
							});
						}
						html! {<>{spans}</>}
					}
				}}
			</td>
		</SpellModalRowRoot>
	}
}

#[function_component]
fn SpellModalRowRoot(SpellModalProps { location, children }: &SpellModalProps) -> Html {
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_browser = modal_dispatcher.callback({
		let location = location.clone();
		move |_| {
			let location = location.clone();
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("spell"),
				content: html! {<SpellModal {location} />},
				..Default::default()
			})
		}
	});
	html! {
		<tr onclick={open_browser}>
			{children.clone()}
		</tr>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SpellModalProps {
	location: SpellLocation,
	#[prop_or_default]
	children: Children,
}
#[function_component]
fn SpellModal(
	SpellModalProps {
		location,
		children: _,
	}: &SpellModalProps,
) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let Some((spell, entry)) = location.get(&state) else {
		log::warn!("Invalid spell at location {location:?}");
		return Html::default();
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{&spell.name}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{spell_content(spell, entry, &state)}
		</div>
	</>}
}

#[function_component]
fn ManagerCasterModal(CasterNameProps { caster_id }: &CasterNameProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let Some(caster) = state.spellcasting().get_caster(caster_id.as_str()) else {
		return html! {<>
			<div class="modal-header">
				<h1 class="modal-title fs-4">{"Not Found: "}{caster_id}</h1>
				<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
			</div>
			<div class="modal-body">
				{"No spellcasting data found."}
			</div>
		</>};
	};

	let max_cantrips = caster.cantrip_capacity(state.persistent());
	let max_spells = caster.spell_capacity(&state);
	let filter = state
		.spellcasting()
		.get_filter(caster.name(), state.persistent())
		.unwrap_or_default();

	let mut num_cantrips = 0;
	let mut num_spells = 0;
	let mut num_all_selections = 0;
	if let Some(selections) = state.persistent().selected_spells.get(caster.name()) {
		num_cantrips = selections.num_cantrips;
		num_spells = selections.num_spells;
		num_all_selections = selections.len();
	}
	let mut selected_spells = Vec::with_capacity(num_all_selections);
	if let Some(iter_selected) = state
		.persistent()
		.selected_spells
		.iter_caster(caster.name())
	{
		for spell in iter_selected {
			// Since we are only processing one caster per section, we can assume the
			// selected spells have already been sorted (rank then name) when they were loaded from kdl.
			selected_spells.push(spell);
		}
	}
	let caster_info = ActionCasterInfo {
		id: caster_id.clone(),
		max_cantrips,
		max_spells,
	};

	// TODO: Display modifier/atk bonus/save dc and how they are calculated for this caster.
	// TODO: Display restriction info for the caster's spell list.
	// TODO: Display rules for when spells can be selected or swapped out.
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{caster.name().clone()}{" Spellcasting"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div>
				<CollapsableCard
					id={"selected-spells"}
					header_content={{html! { {"Selected Spells"} }}}
					body_classes={"spell-list selected"}
				>
					{selected_spells.into_iter().map(|spell| {
						let action = html! {<SpellListAction
							section={SpellListSection::Selected}
							caster={caster_info.clone()}
							spell_id={spell.id.unversioned()}
							rank={spell.rank}
						/>};
						spell_list_item("selected", &state, spell, &caster.spell_entry, action)
					}).collect::<Vec<_>>()}
				</CollapsableCard>
				<CollapsableCard
					id={"available-spells"}
					header_content={{html! { {"Available Spells"} }}}
					body_classes={"spell-list available"}
				>
					<AvailableSpellList
						header_addon={HeaderAddon::from({
							let caster_info = caster_info.clone();
							move |spell: &Spell| -> Html {
								html! {
									<SpellListAction
										caster={caster_info.clone()}
										section={SpellListSection::Available}
										spell_id={spell.id.unversioned()}
										rank={spell.rank}
									/>
								}
							}
						})}
						criteria={filter.as_criteria()}
						entry={caster.spell_entry.clone()}
					/>
				</CollapsableCard>
			</div>
		</div>
		<div class="modal-footer">
			<SpellCapacity name={"Cantrips"} num={num_cantrips} max={max_cantrips} />
			<SpellCapacity name={"Spells"} num={num_spells} max={max_spells} />
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct SpellCapacityProps {
	name: AttrValue,
	num: usize,
	max: usize,
}
#[function_component]
fn SpellCapacity(SpellCapacityProps { name, num, max }: &SpellCapacityProps) -> Html {
	let mut classes = classes!("alert");
	if num < max {
		classes.push("alert-warning");
	}
	html! {
		<div class={classes} role="alert">
			{name}{": "}{num}{" / "}{max}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SpellListActionProps {
	caster: ActionCasterInfo,
	section: SpellListSection,
	spell_id: SourceId,
	rank: u8,
}
#[derive(Clone, PartialEq)]
struct ActionCasterInfo {
	id: AttrValue,
	max_cantrips: usize,
	max_spells: usize,
}
#[derive(Clone, PartialEq)]
enum SpellListSection {
	Selected,
	Available,
}
#[function_component]
fn SpellListAction(
	SpellListActionProps {
		caster: info,
		section,
		spell_id,
		rank,
	}: &SpellListActionProps,
) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let Some(caster) = state.spellcasting().get_caster(info.id.as_str()) else { return Html::default(); };

	let mut can_select_more = true;
	if let Some(selections) = state.persistent().selected_spells.get(&info.id) {
		can_select_more = match rank {
			0 => selections.num_cantrips < info.max_cantrips,
			_ => selections.num_spells < info.max_spells,
		};
	}

	let is_selected = state
		.persistent()
		.selected_spells
		.has_selected(&info.id, &spell_id);

	let mut classes = classes!("btn", "btn-xs", "select");
	let mut disabled = false;
	match section {
		SpellListSection::Selected => {
			classes.push("btn-outline-theme");
		}
		SpellListSection::Available => match is_selected {
			true => {
				classes.push("btn-theme");
			}
			false => match can_select_more {
				true => {
					classes.push("btn-outline-theme");
				}
				false => {
					classes.push("btn-outline-secondary");
					disabled = true;
				}
			},
		},
	}

	let action_name = match (caster.kind(), rank, is_selected) {
		(_, 0, false) => "Learn",
		(_, 0, true) => "Delete",
		(CasterKind::Known, _, false) => "Learn",
		(CasterKind::Known, _, true) => "Delete",
		(CasterKind::Prepared, _, false) => "Prepare",
		(CasterKind::Prepared, _, true) => match section {
			SpellListSection::Selected => "Unprepare",
			SpellListSection::Available => "Prepared",
		},
	};

	let select_spell = use_typed_fetch_callback(
		"Select Spell".into(),
		Callback::from({
			let caster_id = info.id.clone();
			state.new_dispatch(move |spell: Spell, persistent| {
				persistent.selected_spells.insert(&caster_id, spell);
				MutatorImpact::None // TODO: maybe recompile when spells are added because of bonuses to spell attacks and other mutators?
			})
		}),
	);
	let deselect_spell = state.new_dispatch({
		let caster_id = info.id.clone();
		move |spell_id: SourceId, persistent| {
			persistent.selected_spells.remove(&caster_id, &spell_id);
			MutatorImpact::None
		}
	});
	let onclick = Callback::from({
		let spell_id = spell_id.clone();
		move |evt: MouseEvent| {
			evt.stop_propagation();
			let target = match is_selected {
				true => &deselect_spell,
				false => &select_spell,
			};
			target.emit(spell_id.clone());
		}
	});
	let onclick = (!disabled).then_some(onclick);

	html! {
		<button type="button" class={classes} {disabled} {onclick}>{action_name}</button>
	}
}

fn spell_list_item(
	section_id: &str,
	state: &CharacterHandle,
	spell: &Spell,
	entry: &SpellEntry,
	action: Html,
) -> Html {
	let collapse_id = format!("{section_id}-{}", spell.id.ref_id());
	let can_ritual_cast = spell.casting_time.ritual && {
		let classified = entry.classified_as.as_ref();
		let caster = classified
			.map(|id| state.spellcasting().get_caster(id))
			.flatten();
		let ritual_casting = caster
			.map(|caster| caster.ritual_capability.as_ref())
			.flatten();
		match ritual_casting {
			None => false,
			Some(RitualCapability {
				selected_spells,
				available_spells,
			}) => *available_spells || *selected_spells,
		}
	};
	// TODO: Tooltips for ritual & concentration icons
	html! {
		<div class="spell mb-1">
			<div class="header mb-1">
				<button
					role="button" class={"collapse_trigger arrow_left collapsed"}
					data-bs-toggle="collapse"
					data-bs-target={format!("#{collapse_id}")}
				>
					{spell.name.clone()}
					{can_ritual_cast.then(|| html!(
						<Glyph tag="div" classes={"ritual ms-1 my-auto"} />
					)).unwrap_or_default()}
					{spell.duration.concentration.then(|| html!(
						<Glyph tag="div" classes={"concentration ms-1 my-auto"} />
					)).unwrap_or_default()}
					<span class="spell_rank_suffix">
						{"("}
						{match spell.rank {
							0 => "Cantrip".into(),
							n => format!("{n}{}", rank_suffix(n))
						}}
						{")"}
					</span>
				</button>
				{action}
			</div>
			<div class="collapse mb-2" id={collapse_id}>
				<div class="card">
					<div class="card-body px-2 py-1">
						{spell_content(&spell, entry, state)}
					</div>
				</div>
			</div>
		</div>
	}
}

fn spell_content(spell: &Spell, entry: &SpellEntry, state: &CharacterHandle) -> Html {
	use crate::{
		components::{Tag, Tags},
		system::dnd5e::data::AreaOfEffect,
	};
	use spell::{CastingDuration, DurationKind};
	let mut sections = Vec::new();
	sections.push(html! {
		<div class="property">
			<strong>{"Rank:"}</strong>
			{match spell.rank {
				0 => html! { "Cantrip" },
				n => html! {<>{n}{rank_suffix(n)}{" Level"}</>},
			}}
		</div>
	});
	if let Some(school) = &spell.school_tag {
		sections.push(html! {
			<div class="property">
				<strong>{"School:"}</strong>
				{school}
			</div>
		});
	}
	sections.push(html! {
		<div class="property">
			<strong>{"Casting Time:"}</strong>
			{match &spell.casting_time.duration {
				CastingDuration::Action => format!("1 action"),
				CastingDuration::Bonus => format!("1 bonus action"),
				CastingDuration::Reaction(None) => format!("1 reaction"),
				CastingDuration::Reaction(Some(trigger)) => format!("1 reaction ({trigger})"),
				CastingDuration::Unit(amt, kind) => format!("{amt} {kind}"),
			}}
			{spell.casting_time.ritual.then(|| html! { {" (ritual)"} }).unwrap_or_default()}
		</div>
	});
	sections.push(html! {
		<div class="property">
			<strong>{"Range:"}</strong>
			{match entry.range.as_ref().unwrap_or(&spell.range) {
				spell::Range::OnlySelf => html!("Self"),
				spell::Range::Touch => html!("Touch"),
				spell::Range::Unit { distance, unit } => html! {<>{distance}{" "}{unit}</>},
				spell::Range::Sight => html!("Sight"),
				spell::Range::Unlimited => html!("Unlimited"),
			}}
		</div>
	});
	if let Some(area_of_effect) = &spell.area_of_effect {
		sections.push(html! {
			<div class="property">
				<strong>{"Area of Effect:"}</strong>
				{match area_of_effect {
					AreaOfEffect::Cone { length } => html!{<>{length}{"ft. Cone"}</>},
					AreaOfEffect::Cube { size } => html!{<>{size}{"ft. Cube"}</>},
					AreaOfEffect::Cylinder { radius, height } => html!{<>{radius}{"ft. x "}{height}{"ft. Cylinder"}</>},
					AreaOfEffect::Line { width, length } => html!{<>{width}{"ft. x "}{length}{"ft. Line"}</>},
					AreaOfEffect::Sphere { radius } => html!{<>{radius}{"ft. Sphere"}</>},
				}}
			</div>
		});
	}
	sections.push(html! {
		<div class="property">
			<strong>{"Duration:"}</strong>
			{match &spell.duration.kind {
				DurationKind::Instantaneous => html!("Instantaneous"),
				DurationKind::Unit(amt, kind) => html! {<>{amt}{" "}{kind}</>},
				DurationKind::Special => html!("Special"),
			}}
			{spell.duration.concentration.then(|| html!(" (requires concentration)")).unwrap_or_default()}
		</div>
	});
	if !spell.tags.is_empty() {
		sections.push(html! {
			<div class="property d-inline-flex">
				<strong>{"Tags:"}</strong>
				<Tags>
					{spell.tags.iter().map(|tag| html! {
						<Tag>{tag}</Tag>
					}).collect::<Vec<_>>()}
				</Tags>
			</div>
		});
	}

	if let Some(module) = &spell.id.module {
		sections.push(html! {
			<div class="property">
				<strong>{"Source:"}</strong>
				{match module {
					ModuleId::Local { name } => format!("{name} (local)"),
					ModuleId::Github { user_org, repository } => format!("{user_org}:{repository} (github)"),
				}}
			</div>
		});
	}
	if let Some(version) = &spell.id.version {
		sections.push(html! {
			<div class="property">
				<strong>{"Version:"}</strong>
				{version}
			</div>
		});
	}

	let mut component_items = Vec::new();
	if spell.components.verbal {
		component_items.push(html!("Verbal"));
	}
	if spell.components.somatic {
		component_items.push(html!("Somatic"));
	}
	for (material, consumed) in &spell.components.materials {
		component_items.push(html! {
			<span>
				{"Material: "}
				{material}
				{consumed.then(|| " (consumed)").unwrap_or_default()}
			</span>
		});
	}
	sections.push(html! {
		<div class="property">
			<strong>{"Components:"}</strong>
			<ul>
				{component_items.into_iter().map(|entry| html! {<li>{entry}</li>}).collect::<Vec<_>>()}
			</ul>
		</div>
	});

	let desc = {
		let modifier = state.ability_modifier(entry.ability, None);
		let prof_bonus = state.proficiency_bonus();
		let caster_args = std::collections::HashMap::from([
			("{CasterMod}".into(), format!("{:+}", modifier)),
			("{CasterAtk}".into(), format!("{:+}", modifier + prof_bonus)),
			(
				"{CasterDC}".into(),
				format!("{}", 8 + modifier + prof_bonus),
			),
		]);
		let desc = spell
			.description
			.clone()
			.evaluate_with(state, Some(caster_args));
		desc.sections
			.into_iter()
			.map(|section| {
				html! {
					<DescriptionSection {section} show_selectors={false} />
				}
			})
			.collect::<Vec<_>>()
	};

	html! {<>
		{sections}
		<div class="hr my-2" />
		{desc}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
pub struct AvailableSpellListProps {
	pub criteria: Option<crate::database::app::Criteria>,
	pub entry: SpellEntry,
	pub header_addon: HeaderAddon,
}
#[derive(Clone)]
pub struct HeaderAddon(std::sync::Arc<dyn Fn(&Spell) -> Html>);
impl PartialEq for HeaderAddon {
	fn eq(&self, other: &Self) -> bool {
		std::sync::Arc::ptr_eq(&self.0, &other.0)
	}
}
impl<F> From<F> for HeaderAddon
where
	F: Fn(&Spell) -> Html + 'static,
{
	fn from(value: F) -> Self {
		Self(std::sync::Arc::new(value))
	}
}
#[function_component]
pub fn AvailableSpellList(props: &AvailableSpellListProps) -> Html {
	use yew_hooks::{use_async_with_options, UseAsyncOptions};
	log::debug!(target: "ui", "Render available spells");
	let state = use_context::<CharacterHandle>().unwrap();
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let load_data = use_async_with_options(
		{
			let AvailableSpellListProps {
				criteria,
				entry,
				header_addon,
			} = props.clone();
			let state = state.clone();
			async move {
				// TODO: Available spells section wont have togglable filters for max_rank or restriction,
				// BUT when custom feats are a thing, users should be able to add a feat which grants them
				// access to a spell that isnt in their normal class list.
				// e.g. feat which allows the player to have access to a spell
				// 			OR a feat which grants a spell as always prepared.

				let mut sorted_info = Vec::<(String, u8)>::new();
				let mut htmls = Vec::new();

				//let parsing_time = wasm_timer::Instant::now();
				let mut stream =
					FindRelevantSpells::new(database.clone(), &system_depot, criteria.clone());
				while let Some(spell) = stream.next().await {
					// Insertion sort by rank & name
					let idx = sorted_info
						.binary_search_by(|(name, rank)| {
							rank.cmp(&spell.rank).then(name.cmp(&spell.name))
						})
						.unwrap_or_else(|err_idx| err_idx);
					let info = (spell.name.clone(), spell.rank);
					let html = {
						let addon = (header_addon.0)(&spell);
						spell_list_item("relevant", &state, &spell, &entry, addon)
					};
					sorted_info.insert(idx, info);
					htmls.insert(idx, html);
				}

				//log::debug!("Finding and constructing spells took {}s", parsing_time.elapsed().as_secs_f32());
				Ok(html! {<>{htmls}</>}) as Result<Html, ()>
			}
		},
		UseAsyncOptions::default(),
	);
	// PERF: The relevant spell list could be searched when the character is opened, instead of doing it every time the modal is opened
	if yew_hooks::use_is_first_mount() {
		load_data.run();
	}

	// TODO: Search bar for available spells section
	html! {<>
		{match (load_data.loading, &load_data.data) {
			(false, None) => html! {"Spells not loaded"},
			(true, _) => html!(<Spinner />),
			(false, Some(data)) => data.clone(),
		}}
	</>}
}

struct FindRelevantSpells {
	pending_query: Option<
		Pin<Box<dyn futures_util::Future<Output = Result<QueryDeserialize<Spell>, idb::Error>>>>,
	>,
	query: Option<QueryDeserialize<Spell>>,
}
impl FindRelevantSpells {
	fn new(
		database: Database,
		system_depot: &system::Depot,
		criteria: Option<crate::database::app::Criteria>,
	) -> Self {
		use crate::system::core::System;
		let pending_query = database.query_typed::<Spell>(
			DnD5e::id(),
			system_depot.clone(),
			criteria.map(|criteria| criteria.into()),
		);
		Self {
			pending_query: Some(Box::pin(pending_query)),
			query: None,
		}
	}
}
impl FindRelevantSpells {
	fn _pending_delay(cx: &mut std::task::Context<'_>, millis: u32) -> std::task::Poll<Html> {
		use gloo_timers::callback::Timeout;
		let waker = cx.waker().clone();
		if millis > 0 {
			Timeout::new(millis, move || waker.wake()).forget();
		} else {
			waker.wake();
		}
		std::task::Poll::Pending
	}
}
impl futures_util::Stream for FindRelevantSpells {
	type Item = Spell;

	fn poll_next(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Option<Self::Item>> {
		use std::task::Poll;
		if let Some(mut pending) = self.pending_query.take() {
			match pending.poll_unpin(cx) {
				Poll::Pending => {
					self.pending_query = Some(pending);
					return Poll::Pending;
				}
				Poll::Ready(Err(_db_error)) => {
					return Poll::Ready(None);
				}
				Poll::Ready(Ok(query)) => {
					self.query = Some(query);
				}
			}
		}
		loop {
			let Some(mut query) = self.query.take() else { return Poll::Ready(None); };
			// Poll to see if the next spell in the stream is available
			let Poll::Ready(spell) = query.poll_next_unpin(cx) else {
				// still pending, return the query to this struct for next poll.
				self.query = Some(query);
				return Poll::Pending;
			};
			// If there is no spell, the stream is finished and this future is complete.
			let Some(spell) = spell else {
				return Poll::Ready(None);
			};
			// There is a spell, so return the query here for the next loop or poll.
			self.query = Some(query);
			return Poll::Ready(Some(spell));
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
struct UseSpellButtonProps {
	kind: UseSpell,
}
#[derive(Clone, PartialEq)]
enum UseSpell {
	AtWill,
	RitualOnly,
	Slot {
		spell_rank: u8,
		slot_rank: u8,
		slots: Option<(usize, usize)>,
	},
	LimitedUse {
		uses_consumed: u32,
		max_uses: u32,
		data_path: Option<std::path::PathBuf>,
	},
}
#[function_component]
fn UseSpellButton(UseSpellButtonProps { kind }: &UseSpellButtonProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	match kind {
		UseSpell::AtWill => html! {
			<div class="text-center" style="font-size: 9px; font-weight: 700; color: var(--bs-gray-600);">
				{"AT"}<br />{"WILL"}
			</div>
		},
		UseSpell::RitualOnly => html! {
			<div class="text-center" style="font-size: 9px; font-weight: 700; color: var(--bs-gray-600);">
				{"RITUAL"}<br />{"ONLY"}
			</div>
		},
		UseSpell::Slot {
			spell_rank,
			slot_rank,
			slots,
		} => {
			let can_cast = slots
				.as_ref()
				.map(|(consumed, max)| consumed < max)
				.unwrap_or(false);
			let onclick = state.new_dispatch({
				let consumed_slots = slots
					.as_ref()
					.map(|(consumed, _max)| *consumed)
					.unwrap_or(0);
				let slot_rank = *slot_rank;
				move |evt: MouseEvent, persistent| {
					evt.stop_propagation();
					if can_cast {
						let data_path = persistent.selected_spells.consumed_slots_path(slot_rank);
						persistent.set_selected_value(&data_path, (consumed_slots + 1).to_string());
					}
					MutatorImpact::None
				}
			});
			// TODO: Revisit when spell panel rows are more fleshed out. This upcast rank span thing is not
			// very well formated/displayed, esp with such a small text size.
			let upcast_span = (slot_rank > spell_rank).then(|| html! {
				<span class="d-flex position-absolute" style="left: 1px; right: 0; top: -8px;">
					<span style="align-items: flex-start; background-color: #1c9aef; border: 1px solid hsla(0,0%,100%,.5); border-radius: 2px; color: #fff; display: flex; font-size: 8px; line-height: 1; padding: 1px 3px;">
						{*spell_rank}
						<span style="font-size: 6px;">{rank_suffix(*spell_rank)}</span>
					</span>
				</span>
			});
			let mut btn_classes = classes!("btn", "btn-xs", "px-1", "w-100");
			btn_classes.push(match can_cast {
				true => classes!("btn-theme"),
				false => classes!("btn-outline-theme", "disabled"),
			});
			html! {
				<button class={btn_classes} {onclick}>
					<div class="position-relative">
						{upcast_span.unwrap_or_default()}
						{"Cast"}
					</div>
				</button>
			}
		}
		UseSpell::LimitedUse {
			uses_consumed,
			max_uses,
			data_path,
		} => {
			let onclick = match data_path {
				None => Callback::default(),
				Some(path) => state.new_dispatch({
					let uses_consumed = *uses_consumed;
					let key = path.clone();
					move |evt: MouseEvent, persistent| {
						evt.stop_propagation();
						let uses_consumed = uses_consumed + 1;
						persistent.set_selected_value(&key, uses_consumed.to_string());
						MutatorImpact::None
					}
				}),
			};
			let uses_remaining = max_uses.saturating_sub(*uses_consumed);
			html! {
				<button class="btn btn-theme btn-xs px-1 w-100" {onclick} disabled={uses_consumed >= max_uses}>
					{"Use"}
					<span class="ms-1 d-none" style="font-size: 9px; color: var(--bs-gray-600);">{format!("({uses_remaining}/{max_uses})")}</span>
				</button>
			}
		}
	}
}
