use std::collections::BTreeMap;
use crate::{
	components::modal,
	system::dnd5e::{components::SharedCharacter, DnD5e, data::proficiency},
};
use itertools::Itertools;
use yew::prelude::*;

#[function_component]
pub fn Spells() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let mut entries = Vec::new();
	for (spell_id, _) in state.spellcasting().prepared_spells() {
		entries.push(html! {<div>
			{spell_id.to_string()}
		</div>});
	}
	let mut sections = (0..=9).map(|slot: u8| (slot, SectionProps::default())).collect::<BTreeMap<_, _>>();
	if let Some(slots) = state.spellcasting().spell_slots(&*state) {
		for (rank, slot_count) in slots {
			let section = sections.get_mut(&rank).expect(&format!("Spell rank {rank} is not supported by UI, must be in the range of [0, 9]."));
			section.slot_count = Some(slot_count);
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
				rank => format!("{rank}{}", format!("{suffix} {rank_text}").to_case(Case::Upper)),
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
			let available_spells: Vec<()> = Vec::new();
			if available_spells.is_empty() && (slots.is_none() || rank == 0) {
				continue;
			}

			let contents = match available_spells.is_empty() {
				true => html! { <p class="empty-note mx-4">{empty_note}</p> },
				false => Html::default(),
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

	let open_browser = Callback::from(|_| {});

	let feature_stats = state.spellcasting().has_casters().then(|| {
		use unzip_n::unzip_n;
		unzip_n!(4);
		let (names, modifier, atk_bonus, save_dc) = state.spellcasting().iter_casters().map(|caster| {
			let name = caster.name().clone();
			let modifier = state.ability_modifier(caster.ability, None);
			let atk_bonus = state.ability_modifier(caster.ability, Some(proficiency::Level::Full));
			let save_dc = 8 + atk_bonus;
			(name, modifier, atk_bonus, save_dc)
		}).unzip_n_vec();
		let names = names.into_iter().map(|name| {
			// TODO: Small buttons to open the spellcasting modal for each feature (to display information about spellcasting, not spell management)
			html! {
				<span>{name}</span>
			}
		});
		let names = Itertools::intersperse(names, html! { <span class="mx-1">{"/"}</span> }).collect::<Vec<_>>();
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
					{format!("Spell Slots: {:?}", state.spellcasting().spell_slots(&*state))}
				</div>
				<div>
					{state.spellcasting().iter_casters().map(|caster| {
						html! {
							<div>
								<strong>{caster.name().clone()}</strong>
								<div>
									{format!("Restriction: {:?}", caster.restriction)}
								</div>
								<div>
									{format!("Cantrip Capacity: {:?}", caster.cantrip_capacity(state.persistent()))}
								</div>
								{caster.cantrip_data_path().map(|key| {
									html! { <div>{format!("Cantrips: {:?}", state.get_selections_at(&key))}</div> }
								}).unwrap_or_default()}
								<div>{format!("Spell Capacity: {:?}", caster.spell_capacity(&state))}</div>
								<div>{format!("Max Level: {:?}", caster.max_spell_rank(&state))}</div>
								<div>{format!("Spells: {:?}", state.get_selections_at(&caster.spells_data_path()))}</div>
							</div>
						}
					}).collect::<Vec<_>>()}
				</div>
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

#[derive(Clone, PartialEq, Properties, Default)]
struct SectionProps {
	slot_count: Option<usize>,
}
