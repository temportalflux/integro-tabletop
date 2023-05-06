use crate::{
	components::modal,
	system::dnd5e::{
		components::{editor::CollapsableCard, SharedCharacter},
		data::{proficiency, Spell},
		DnD5e,
	},
};
use itertools::Itertools;
use std::collections::BTreeMap;
use yew::prelude::*;

#[function_component]
pub fn Spells() -> Html {
	static MAX_SPELL_RANK: u8 = 9;
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_browser = modal_dispatcher.callback(|_| {
		modal::Action::Open(modal::Props {
			centered: true,
			scrollable: true,
			root_classes: classes!("spells", "browse"),
			content: html! {<BrowseModal />},
			..Default::default()
		})
	});

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
			section.slot_count = Some(slot_count);
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
		use convert_case::{Case, Casing};
		let mut html = Vec::new();
		for (rank, section_props) in sections {
			let suffix = match rank {
				1 => "st",
				2 => "nd",
				3 => "rd",
				4..=9 => "th",
				_ => "",
			};
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

			let slots = section_props.slot_count.as_ref().map(|count| {
				html! {
					<div class="slots">
						{(0..*count).map(|_| html! {
							<input class={"form-check-input slot"} type="checkbox" />
						}).collect::<Vec<_>>()}
						<span class="ms-1">{"SLOTS"}</span>
					</div>
				}
			});
			if section_props.spells.is_empty() && (slots.is_none() || rank == 0) {
				continue;
			}

			let contents = match section_props.spells.is_empty() {
				true => html! { <p class="empty-note mx-4">{empty_note}</p> },
				false => {
					let rows = section_props
						.spells
						.iter()
						.map(|entry| {
							html! {
								<div class="d-flex">
									<div class="me-2">{entry.spell.name.clone()}</div>
									<div class="me-2">{entry.spell.id.to_string()}</div>
								</div>
							}
						})
						.collect::<Vec<_>>();
					html! {
						<div>
							{rows}
						</div>
					}
				}
			};
			html.push(html! {
				<div class="spell-section mb-2">
					<div class="header">
						<div class="title">{title}</div>
						{slots.unwrap_or_default()}
					</div>
					{contents}
				</div>
			});
		}
		html
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
		let names = names.into_iter().map(|name| {
			// TODO: Small buttons to open the spellcasting modal for each feature (to display information about spellcasting, not spell management)
			html! {
				<span>{name}</span>
			}
		});
		let names = Itertools::intersperse(names, html! { <span class="mx-1">{"/"}</span> })
			.collect::<Vec<_>>();
		let modifier = modifier.into_iter().map(|v| format!("{v:+}")).join(" / ");
		let atk_bonus = atk_bonus.into_iter().map(|v| format!("{v:+}")).join(" / ");
		let save_dc = save_dc.into_iter().map(|v| format!("{v}")).join(" / ");
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
				<button type="button" class="btn btn-outline-theme" onclick={open_browser}>{"Manage Spells"}</button>
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

#[derive(Clone, PartialEq, Default)]
struct SectionProps<'c> {
	slot_count: Option<usize>,
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
			a.spell
				.rank
				.cmp(&spell.rank)
				.then(a.spell.name.cmp(&spell.name))
		});
		let idx = idx.unwrap_or_else(|e| e);
		self.spells.insert(idx, SpellEntry { spell, caster_name });
	}
}

#[function_component]
pub fn BrowseModal() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let mut sections = Vec::new();
	for caster in state.spellcasting().iter_casters() {
		let restriction = &caster.restriction;

		let max_cantrips = caster.cantrip_capacity(state.persistent());
		let max_spells = caster.spell_capacity(&state);
		let max_spell_rank = caster.max_spell_rank(&state);
		let mut num_cantrips = 0;
		let mut num_spells = 0;
		let num_all_selections = state
			.persistent()
			.selected_spells
			.len_caster(caster.name())
			.unwrap_or(0);
		let mut selected_spells = Vec::with_capacity(num_all_selections);
		if let Some(iter_selected) = state
			.persistent()
			.selected_spells
			.iter_caster(caster.name())
		{
			for spell in iter_selected {
				match spell.rank {
					0 => num_cantrips += 1,
					_ => num_spells += 1,
				}
				// Since we are only processing one caster per section, we can assume the
				// selected spells have already been sorted (rank then name) when they were loaded from kdl.
				selected_spells.push(spell);
			}
		}

		sections.push(html! {
			<div>
				<div>{caster.name().clone()}</div>

				<div>
					<CollapsableCard
						id={"selected-spells"}
						header_content={{html! { {"Selected Spells"} }}}
						body_classes={"spell-list selected"}
					>
						<div>{format!("Cantrips: {num_cantrips} / {max_cantrips}")}</div>
						<div>{format!("Spells: {num_spells} / {max_spells}")}</div>
						{selected_spells.into_iter().map(|spell| {
							html! {
								<div>
									{spell.name.clone()}
									{format!(" ({})", spell.rank)}
								</div>
							}
						}).collect::<Vec<_>>()}
					</CollapsableCard>
					<CollapsableCard
						id={"available-spells"}
						header_content={{html! { {"Available Spells"} }}}
						body_classes={"spell-list available"}
					>
						{"List of all spells that can be selected (known or prepared)"}
						<div>{format!("Restriction: {restriction:?}")}</div>
						<div>{format!("Max Spell Level: {max_spell_rank:?}")}</div>
					</CollapsableCard>
				</div>
			</div>
		});
	}
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Manage Spells"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{sections}
		</div>
	</>}
}
