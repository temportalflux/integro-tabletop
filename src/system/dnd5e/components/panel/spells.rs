use crate::{
	components::modal,
	system::{
		core::SourceId,
		dnd5e::{
			components::{editor::CollapsableCard, SharedCharacter},
			data::{character::spellcasting::Restriction, proficiency, Spell},
			DnD5e,
		},
	},
};
use itertools::Itertools;
use std::collections::BTreeMap;
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
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();

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
		let names = names.into_iter().map(|caster_id| {
			html! {
				<ManageCasterButton {caster_id} />
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
		<button type="button" class="btn btn-xs btn-outline-theme" onclick={open_browser}>
			{caster_id.clone()}
		</button>
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

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Manage Spells"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div>{caster.name().clone()}</div>
			<div>{format!("Cantrips: {num_cantrips} / {max_cantrips}")}</div>
			<div>{format!("Spells: {num_spells} / {max_spells}")}</div>
			<div>
				<CollapsableCard
					id={"selected-spells"}
					header_content={{html! { {"Selected Spells"} }}}
					body_classes={"spell-list selected"}
				>
					{selected_spells.into_iter().map(|spell| {
						let action = html! { <button type="button" class="btn btn-xs btn-outline-theme select">{"Select"}</button> };
						spell_list_item(spell, action)
					}).collect::<Vec<_>>()}
				</CollapsableCard>
				<CollapsableCard
					id={"available-spells"}
					header_content={{html! { {"Available Spells"} }}}
					body_classes={"spell-list available"}
				>
					<AvailableSpellList
						restriction={caster.restriction.clone()}
						max_rank={max_spell_rank}
					/>
				</CollapsableCard>
			</div>
		</div>
	</>}
}

fn spell_list_item(spell: &Spell, action: Html) -> Html {
	let collapse_id = spell.id.ref_id();
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
						<div>{"spell content for "}{spell.name.clone()}</div>
						<div>{spell.id.clone()}</div>
					</div>
				</div>
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SpellListFilter {
	restriction: Restriction,
	max_rank: Option<u8>,
}
#[function_component]
fn AvailableSpellList(props: &SpellListFilter) -> Html {
	use yew_hooks::{use_async_with_options, UseAsyncOptions};
	log::debug!(target: "ui", "Render available spells");
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let load_data = use_async_with_options(
		{
			let filter = props.clone();
			let system = system.clone();
			async move {
				// TODO: Available spells section wont have togglable filters for max_rank or restriction,
				// BUT when custom feats are a thing, users should be able to add a feat which grants them
				// access to a spell that isnt in their normal class list.
				// e.g. feat which allows the player to have access to a spell
				// 			OR a feat which grants a spell as always prepared.
				Ok(FindRelevantSpells::new(system.clone(), filter).await) as Result<Html, ()>
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
	filter: SpellListFilter,
	system: UseStateHandle<DnD5e>,
	all_ids: Vec<SourceId>,
	sorted_info: Vec<(String, u8)>,
	spell_htmls: Option<Vec<Html>>,
}
impl FindRelevantSpells {
	fn new(system: UseStateHandle<DnD5e>, filter: SpellListFilter) -> Self {
		let all_ids = system.spells.keys().cloned().collect::<Vec<_>>();
		let sorted_info = Vec::with_capacity(all_ids.len());
		let spell_htmls = Some(Vec::with_capacity(all_ids.len()));
		Self {
			system,
			all_ids,
			filter,
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
			if let Some(max_rank) = self.filter.max_rank {
				if spell.rank > max_rank {
					return Self::pending_delay(cx, 0);
				}
			}
			for tag in &self.filter.restriction.tags {
				if !spell.tags.contains(tag) {
					return Self::pending_delay(cx, 0);
				}
			}

			// Insertion sort by rank & name
			let idx = self
				.sorted_info
				.binary_search_by(|(name, rank)| rank.cmp(&spell.rank).then(name.cmp(&spell.name)))
				.unwrap_or_else(|err_idx| err_idx);

			let info = (spell.name.clone(), spell.rank);
			let html = {
				let action = html! { <button type="button" class="btn btn-xs btn-outline-theme select">{"Select"}</button> };
				spell_list_item(spell, action)
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
