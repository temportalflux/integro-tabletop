use crate::{
	components::{modal, stop_propagation},
	system::{
		core::{ModuleId, SourceId},
		dnd5e::{
			components::{editor::CollapsableCard, SharedCharacter},
			data::{
				character::spellcasting::{CasterKind, Restriction},
				proficiency, spell, Spell,
			},
			DnD5e,
		},
	},
	utility::InputExt,
};
use convert_case::{Case, Casing};
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};
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
	static MAX_SPELL_RANK: u8 = 9;
	let state = use_context::<SharedCharacter>().unwrap();

	let mut entries = Vec::new();
	for (spell_id, _) in state.spellcasting().prepared_spells() {
		entries.push(html! {<div>
			{spell_id.to_string()}
		</div>});
	}
	let mut sections = (0..=MAX_SPELL_RANK)
		.map(|slot: u8| (slot, SectionProps::default()))
		.collect::<BTreeMap<_, _>>();
	if let Some(slots) = state.spellcasting().spell_slots(&*state) {
		for (rank, slot_count) in slots {
			let section = sections.get_mut(&rank).expect(&format!(
				"Spell rank {rank} is not supported by UI, must be in the range of [0, {MAX_SPELL_RANK}]."
			));
			let consumed_slots = state
				.persistent()
				.selected_spells
				.consumed_slots(rank)
				.unwrap_or(0);
			section.slot_count = Some((consumed_slots, slot_count));
		}
	}
	for (spell, caster_names) in state.persistent().selected_spells.iter_all_casters() {
		let max_rank = match spell.rank {
			0 => 0,
			_ => MAX_SPELL_RANK,
		};
		for section_rank in spell.rank..=max_rank {
			let Some(section) = sections.get_mut(&section_rank) else { continue; };
			if section_rank == 0 || section.slot_count.is_some() {
				for caster_name in caster_names {
					section.insert_spell(spell, caster_name);
				}
			}
		}
	}

	let sections = {
		let mut html = Vec::new();
		for (rank, section_props) in sections {
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
		let names = names.into_iter().map(|caster_id| {
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
				<div>
					<strong>{"Always Prepared:"}</strong>
					{entries}
				</div>
				<div>
					{format!("{:?}", state.spellcasting())}
				</div>
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

#[derive(Clone, PartialEq, Default)]
struct SectionProps<'c> {
	slot_count: Option<(/*consumed*/ usize, /*max*/ usize)>,
	spells: Vec<SpellEntry<'c>>,
}
#[derive(Clone, PartialEq)]
struct SpellEntry<'c> {
	caster_name: &'c String,
	spell: &'c Spell,
}
impl<'c> SectionProps<'c> {
	pub fn insert_spell(&mut self, spell: &'c Spell, caster_name: &'c String) {
		let idx = self.spells.binary_search_by(|a: &SpellEntry<'c>| {
			//a.spell.rank.cmp(&spell.rank)
			a.spell.name.cmp(&spell.name)
		});
		let idx = idx.unwrap_or_else(|e| e);
		self.spells.insert(idx, SpellEntry { spell, caster_name });
	}
}

fn spell_section<'c>(
	state: &'c SharedCharacter,
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
			move |evt: web_sys::Event, persistent, _| {
				let Some(consume_slot) = evt.input_checked() else { return None; };
				let new_consumed_slots = match consume_slot {
					true => consumed_slots.saturating_add(1),
					false => consumed_slots.saturating_sub(1),
				};
				persistent
					.selected_spells
					.set_slots_consumed(rank, new_consumed_slots);
				None
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
						{section_props.spells.iter().map(|entry| {
							spell_row(state, rank, &section_props.slot_count, entry)
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
fn spell_row<'c>(
	state: &'c SharedCharacter,
	section_rank: u8,
	slots: &Option<(usize, usize)>,
	entry: &SpellEntry<'c>,
) -> Html {
	use spell::CastingDuration;

	let use_kind = match entry.spell.rank {
		//0 => UseSpell::LimitedUse { uses_remaining: 3, max_uses: 5 },
		0 => UseSpell::AtWill,
		spell_rank => UseSpell::Slot {
			spell_rank,
			slot_rank: section_rank,
			slots: slots.clone(),
		},
	};

	// TODO: concentration and ritual icons after the spell name
	// TODO: Casting source under the spell name
	// TODO: tooltip for casting time duration
	html! {
		<SpellModalRowRoot caster_id={entry.caster_name.clone()} spell_id={entry.spell.id.clone()}>
			<td onclick={stop_propagation()}><UseSpellButton kind={use_kind} /></td>
			<td>{&entry.spell.name}</td>
			<td>
				{match &entry.spell.casting_time.duration {
					CastingDuration::Action => html!("1A"),
					CastingDuration::Bonus => html!("1BA"),
					CastingDuration::Reaction(_trigger) => html!("1R"),
					CastingDuration::Unit(amt, kind) => html!{<>{amt}{kind.chars().next().unwrap()}</>},
				}}
			</td>
			<td>
				{match &entry.spell.range {
					spell::Range::OnlySelf => html!("Self"),
					spell::Range::Touch => html!("Touch"),
					spell::Range::Unit { distance, unit } => html! {<>{distance}{" "}{unit}</>},
					spell::Range::Sight => html!("Sight"),
					spell::Range::Unlimited => html!("Unlimited"),
				}}
			</td>
			<td>
				{match &entry.spell.check {
					None => html!("--"),
					Some(spell::Check::AttackRoll(_atk_kind)) => {
						match state.spellcasting().get_caster(entry.caster_name) {
							None => html!("--"),
							Some(caster) => {
								let modifier = state.ability_modifier(caster.ability, Some(proficiency::Level::Full));
								html!(format!("{modifier:+}"))
							}
						}
					}
					Some(spell::Check::SavingThrow(ability, fixed_dc)) => {
						let abb_name = ability.abbreviated_name().to_case(Case::Upper);
						match fixed_dc {
							Some(dc) => html!(format!("{abb_name} {dc}")),
							None => match state.spellcasting().get_caster(entry.caster_name) {
								None => html!("--"),
								Some(caster) => {
									let modifier = state.ability_modifier(caster.ability, Some(proficiency::Level::Full));
									let dc = 8 + modifier;
									html!(format!("{abb_name} {dc}"))
								}
							}
						}
					}
				}}
			</td>
			<td>
				{match &entry.spell.damage {
					None => html!("--"),
					Some(damage) => {
						let modifier = match state.spellcasting().get_caster(entry.caster_name) {
							None => 0,
							Some(caster) => state.ability_modifier(caster.ability, Some(proficiency::Level::Full)),
						};
						let upcast_amt = section_rank - entry.spell.rank;
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
fn SpellModalRowRoot(
	SpellModalProps {
		caster_id,
		spell_id,
		children,
	}: &SpellModalProps,
) -> Html {
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_browser = modal_dispatcher.callback({
		let caster_id = caster_id.clone();
		let spell_id = spell_id.clone();
		move |_| {
			let caster_id = caster_id.clone();
			let spell_id = spell_id.clone();
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("spell"),
				content: html! {<SpellModal {caster_id} {spell_id} />},
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
	caster_id: AttrValue,
	spell_id: SourceId,
	#[prop_or_default]
	children: Children,
}
#[function_component]
fn SpellModal(
	SpellModalProps {
		caster_id,
		spell_id,
		children: _,
	}: &SpellModalProps,
) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let Some(spell) = state.persistent().selected_spells.get_spell(caster_id.as_str(), spell_id) else { return Html::default(); };

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{&spell.name}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{spell_content(spell)}
		</div>
	</>}
}

#[function_component]
fn ManagerCasterModal(CasterNameProps { caster_id }: &CasterNameProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
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
	let filter = SpellFilter {
		tags: caster.restriction.tags.iter().cloned().collect(),
		max_rank: caster.max_spell_rank(&state),
		..Default::default()
	};
	let mut num_cantrips = 0;
	let mut num_spells = 0;
	let mut num_all_selections = 0;
	if let Some(selections) = state.persistent().selected_spells.get(Some(caster.name())) {
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
							spell_id={spell.id.clone()}
							rank={spell.rank}
						/>};
						spell_list_item("selected", spell, action)
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
										spell_id={spell.id.clone()}
										rank={spell.rank}
									/>
								}
							}
						})}
						filter={filter.clone()}
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
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let Some(caster) = state.spellcasting().get_caster(info.id.as_str()) else { return Html::default(); };

	let mut can_select_more = true;
	if let Some(selections) = state
		.persistent()
		.selected_spells
		.get(Some(info.id.as_str()))
	{
		can_select_more = match rank {
			0 => selections.num_cantrips < info.max_cantrips,
			_ => selections.num_spells < info.max_spells,
		};
	}

	let is_selected = state
		.persistent()
		.selected_spells
		.has_selected(Some(info.id.as_str()), &spell_id);

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

	let onclick = (!disabled).then({
		|| {
			let caster_id = info.id.clone();
			let spell_id = spell_id.clone();
			let system = system.clone();
			state.new_dispatch(move |evt: MouseEvent, persistent, _| {
				evt.stop_propagation();
				if is_selected {
					persistent.selected_spells.remove(&caster_id, &spell_id);
					None
				} else {
					let Some(spell) = system.spells.get(&spell_id) else { return None; };
					persistent.selected_spells.insert(&caster_id, spell);
					None // TODO: maybe recompile when spells are added because of bonuses to spell attacks and other mutators?
				}
			})
		}
	});

	html! {
		<button type="button" class={classes} {disabled} {onclick}>{action_name}</button>
	}
}

fn spell_list_item(section_id: &str, spell: &Spell, action: Html) -> Html {
	let collapse_id = format!("{section_id}-{}", spell.id.ref_id());
	// TODO: concentration and ritual icons in header section
	html! {
		<div class="spell mb-1">
			<div class="header mb-1">
				<button
					role="button" class={"collapse_trigger arrow_left collapsed"}
					data-bs-toggle="collapse"
					data-bs-target={format!("#{collapse_id}")}
				>
					{spell.name.clone()}
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
						{spell_content(&spell)}
					</div>
				</div>
			</div>
		</div>
	}
}

fn spell_content(spell: &Spell) -> Html {
	use crate::{
		components::{Tag, Tags},
		system::dnd5e::{components::editor::description, data::AreaOfEffect},
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
			{match &spell.range {
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

	html! {<>
		{sections}
		<div class="hr my-2" />
		{description(&spell.description, false)}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
pub struct AvailableSpellListProps {
	pub filter: SpellFilter,
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
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpellFilter {
	/// The spell must already be castable by the provided caster class.
	/// This can be true if the spell contains the class tag OR the spell is in the expanded list
	/// for the caster data (e.g. spellcasting "add_source").
	pub can_cast: Option<String>,
	/// The spell must be of one of these ranks.
	pub ranks: HashSet<u8>,
	/// The spell's rank must be <= this rank.
	pub max_rank: Option<u8>,
	/// The spell must have all of these tags.
	pub tags: HashSet<String>,
}
#[function_component]
pub fn AvailableSpellList(props: &AvailableSpellListProps) -> Html {
	use yew_hooks::{use_async_with_options, UseAsyncOptions};
	log::debug!(target: "ui", "Render available spells");
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let load_data = use_async_with_options(
		{
			let props = props.clone();
			let system = system.clone();
			async move {
				// TODO: Available spells section wont have togglable filters for max_rank or restriction,
				// BUT when custom feats are a thing, users should be able to add a feat which grants them
				// access to a spell that isnt in their normal class list.
				// e.g. feat which allows the player to have access to a spell
				// 			OR a feat which grants a spell as always prepared.
				Ok(FindRelevantSpells::new(system.clone(), props).await) as Result<Html, ()>
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
			(true, _) => html! {
				<div class="spinner-border" role="status">
					<span class="visually-hidden">{"Loading..."}</span>
				</div>
			},
			(false, Some(data)) => data.clone(),
		}}
	</>}
}

struct FindRelevantSpells {
	filter: SpellFilter,
	header_addon: HeaderAddon,
	system: UseStateHandle<DnD5e>,
	all_ids: Vec<SourceId>,
	sorted_info: Vec<(String, u8)>,
	spell_htmls: Option<Vec<Html>>,
}
impl FindRelevantSpells {
	fn new(system: UseStateHandle<DnD5e>, props: AvailableSpellListProps) -> Self {
		let all_ids = system.spells.keys().cloned().collect::<Vec<_>>();
		let sorted_info = Vec::with_capacity(all_ids.len());
		let spell_htmls = Some(Vec::with_capacity(all_ids.len()));
		Self {
			filter: props.filter,
			header_addon: props.header_addon,
			system,
			all_ids,
			sorted_info,
			spell_htmls,
		}
	}
}
impl FindRelevantSpells {
	fn pending_delay(cx: &mut std::task::Context<'_>, millis: u32) -> std::task::Poll<Html> {
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
impl futures::Future for FindRelevantSpells {
	type Output = Html;

	fn poll(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Self::Output> {
		if let Some(spell_id) = self.all_ids.pop() {
			let Some(spell) = self.system.spells.get(&spell_id) else {
				return Self::pending_delay(cx, 0);
			};

			// Filter based on restriction
			if !self.filter.ranks.is_empty() {
				if !self.filter.ranks.contains(&spell.rank) {
					return Self::pending_delay(cx, 0);
				}
			}
			if let Some(max_rank) = self.filter.max_rank {
				if spell.rank > max_rank {
					return Self::pending_delay(cx, 0);
				}
			}
			for tag in &self.filter.tags {
				if !spell.tags.contains(tag) {
					return Self::pending_delay(cx, 0);
				}
			}
			if let Some(caster_class) = &self.filter.can_cast {
				if !spell.tags.contains(caster_class) {
					return Self::pending_delay(cx, 0);
				}
				// TODO: check if the spell is in the expanded spell list,
				// as provided by the AddSource spellcasting mutator.
			}

			// Insertion sort by rank & name
			let idx = self
				.sorted_info
				.binary_search_by(|(name, rank)| rank.cmp(&spell.rank).then(name.cmp(&spell.name)))
				.unwrap_or_else(|err_idx| err_idx);

			let info = (spell.name.clone(), spell.rank);
			let html = {
				let addon = (self.header_addon.0)(spell);
				spell_list_item("relevant", spell, addon)
			};
			drop(spell);
			self.sorted_info.insert(idx, info);

			let mut spell_htmls = self.spell_htmls.take().unwrap();
			spell_htmls.insert(idx, html);
			self.spell_htmls = Some(spell_htmls);

			return Self::pending_delay(cx, 1);
		}

		let spell_htmls = self.spell_htmls.take().unwrap();
		std::task::Poll::Ready(html! {<>{spell_htmls}</>})
	}
}

#[derive(Clone, PartialEq, Properties)]
struct UseSpellButtonProps {
	kind: UseSpell,
}
#[derive(Clone, Copy, PartialEq)]
enum UseSpell {
	AtWill,
	Slot {
		spell_rank: u8,
		slot_rank: u8,
		slots: Option<(usize, usize)>,
	},
	#[allow(dead_code)]
	LimitedUse {
		uses_remaining: u32,
		max_uses: u32,
	},
}
#[function_component]
fn UseSpellButton(UseSpellButtonProps { kind }: &UseSpellButtonProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	match kind {
		UseSpell::AtWill => html! {
			<div class="text-center" style="font-size: 9px; font-weight: 700; color: var(--bs-gray-600);">
				{"AT"}<br />{"WILL"}
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
				move |evt: MouseEvent, persistent, _| {
					evt.stop_propagation();
					if can_cast {
						persistent
							.selected_spells
							.set_slots_consumed(slot_rank, consumed_slots + 1);
					}
					None
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
			uses_remaining,
			max_uses,
		} => {
			html! {
				<button class="btn btn-theme btn-xs px-1 w-100">
					{"Use"}
					<span class="ms-1 d-none" style="font-size: 9px; color: var(--bs-gray-600);">{format!("({uses_remaining}/{max_uses})")}</span>
				</button>
			}
		}
	}
}
