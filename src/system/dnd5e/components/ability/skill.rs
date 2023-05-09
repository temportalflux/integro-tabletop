use crate::{
	bootstrap::components::Tooltip,
	components::modal,
	system::dnd5e::{
		components::{roll::ModifierIcon, SharedCharacter},
		data::{Ability, Skill},
	},
};
use enumset::{EnumSet, EnumSetType};
use multimap::MultiMap;
use yew::prelude::*;

#[derive(Debug, EnumSetType)]
enum Presentation {
	Alphabetical,
	ByAbility,
}
impl Presentation {
	fn display_name(&self) -> &'static str {
		match self {
			Self::Alphabetical => "Alphabetical",
			Self::ByAbility => "By Ability",
		}
	}

	fn icon_html(&self) -> Html {
		match self {
			Self::Alphabetical => html! { <i class="fa-regular fa-a" /> },
			Self::ByAbility => html! { <i class="fa-solid fa-dice-d20" /> },
		}
	}
}
impl std::str::FromStr for Presentation {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Alphabetical" => Ok(Self::Alphabetical),
			"By Ability" => Ok(Self::ByAbility),
			_ => Err(()),
		}
	}
}

#[function_component]
pub fn SkillTable() -> Html {
	let presentation = use_state(|| Presentation::Alphabetical);

	let insert_ability_col_at = match *presentation {
		Presentation::Alphabetical => Some(1),
		Presentation::ByAbility => None,
	};
	let mut headers = vec![
		html! {<th scope="col">{"Prof"}</th>},
		// min-width is to ensure all segments have a wide enough column
		// to account for the largest skill name (so all segments look uniform).
		html! {<th scope="col" style="min-width: 135px;">{"Skill"}</th>},
		html! {<th scope="col">{"Bonus"}</th>},
		html! {<th scope="col">{"Passive"}</th>},
	];
	if let Some(idx) = &insert_ability_col_at {
		headers.insert(*idx, html! {<th scope="col">{"Ability"}</th>});
	}

	let skills: Vec<(Option<Ability>, Vec<Skill>)> = {
		let all = enumset::EnumSet::<Skill>::all().into_iter();
		// Transform the uniform unsorted list of skills based on what categories they should be sorted by.
		let skills = match *presentation {
			// Straight up only-alphabetical has no major category, so all of the skills are lumped together.
			Presentation::Alphabetical => vec![(None, all.into_iter().collect::<Vec<_>>())],
			// Group skills by their core ability.
			Presentation::ByAbility => {
				let skills: MultiMap<Option<Ability>, Skill> =
					all.map(|skill| (Some(skill.ability()), skill)).collect();
				let mut skills = skills.into_iter().collect::<Vec<_>>();
				skills.sort_by(|(ability_a, _), (ability_b, _)| ability_a.cmp(&ability_b));
				skills
			}
		};
		// Always alphabetize the subsections by skill display name
		skills
			.into_iter()
			.map(|(ability, mut skills)| {
				skills.sort_by(|a, b| a.display_name().cmp(&b.display_name()));
				(ability, skills)
			})
			.collect()
	};
	let mut segments = Vec::new();
	for (ability, skills) in skills.into_iter() {
		if let Some(ability) = ability {
			segments.push(html! {
				<div class="text-center" style="width: 100%;">{ability.long_name()}</div>
			});
		}
		segments.push(html! {
			<table class="table table-compact table-striped m-0">
				<thead>
					<tr class="text-center" style="font-size: 0.7rem;">{headers.clone()}</tr>
				</thead>
				<tbody>
					{skills.into_iter().map(|skill| html! {
						<Row {skill} ability_name_col={insert_ability_col_at} />
					}).collect::<Vec<_>>()}
				</tbody>
			</table>
		});
	}

	let on_change_presentation = {
		let state = presentation.clone();
		Callback::from(move |e: MouseEvent| {
			use std::str::FromStr;
			let Some(element) = e.target_dyn_into::<web_sys::HtmlElement>() else { return; };
			let value = element
				.get_attribute("value")
				.map(|s| Presentation::from_str(&s).ok())
				.flatten()
				.unwrap_or(Presentation::Alphabetical);
			state.set(value);
		})
	};

	html! {<>
		<div class="dropdown">
			<button class="btn btn-secondary dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false">
				{presentation.icon_html()}
			</button>
			<div class="dropdown-menu dropdown-menu-end" style="--bs-dropdown-min-width: 0rem;">
				{EnumSet::<Presentation>::all().into_iter().map(|value| html! {
					<a class="dropdown-item" value={value.display_name()} onclick={on_change_presentation.clone()}>
						{value.icon_html()}
						<span style="margin-left: 5px;">{value.display_name()}</span>
					</a>
				}).collect::<Vec<_>>()}
			</div>
		</div>
		{segments}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct RowProps {
	pub skill: Skill,
	pub ability_name_col: Option<usize>,
}
#[function_component]
fn Row(
	RowProps {
		skill,
		ability_name_col,
	}: &RowProps,
) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let proficiency = state.skills().proficiency(*skill);

	let modifier = state.ability_modifier(skill.ability(), Some(*proficiency.value()));
	let passive = 10 + modifier;

	let prof_tooltip = crate::data::as_feature_paths_html_custom(
		proficiency.sources().iter(),
		|(path, prof)| (*prof, path.as_path()),
		|prof, path_str| format!("<div>{} ({})</div>", prof.as_display_name(), path_str),
	);

	let roll_modifiers = {
		use crate::system::dnd5e::data::{
			roll::Modifier,
			character::ModifierMapItem,
		};
		let mut entries = enum_map::EnumMap::<Modifier, Vec<&ModifierMapItem>>::default();
		for (modifier, items) in state.skills().iter_skill_modifiers(*skill) {
			if !items.is_empty() {
				entries[modifier].extend(items.iter());
			}
		}
		let mut entries = entries.into_iter().filter(|(_, items)| !items.is_empty()).collect::<Vec<_>>();
		entries.sort_by_key(|(modifier, _)| *modifier);
		entries
	}.into_iter().map(|(modifier, items)| {
		let tooltip = crate::data::as_feature_paths_html_custom(
			items.into_iter(),
			|item| (item.context.clone(), item.source.as_path()),
			|criteria, path_str| match criteria {
				Some(criteria) => format!("<div>{} ({})</div>", criteria, path_str),
				None => format!("<div>{}</div>", path_str),
			},
		);
		html! {
			<Tooltip tag={"span"} content={tooltip} use_html={true}>
				<span aria-label={format!("{modifier:?}")} style="margin-left: 2px; display: block; height: 16px; width: 16px; vertical-align: middle; margin-top: -2px;">
					<ModifierIcon value={modifier} />
				</span>
			</Tooltip>
		}
	}).collect::<Vec<_>>();

	let mut table_data = vec![
		html! {
			<Tooltip tag={"td"} classes={"text-center"} content={prof_tooltip} use_html={true}>
				{*proficiency.value()}
			</Tooltip>
		},
		html! { <td>
			<div class="d-flex">
				<span class="flex-grow-1">{skill.display_name()}</span>
				{roll_modifiers}
			</div>
		</td> },
		html! { <td class="text-center">{if modifier >= 0 { "+" } else { "-" }}{modifier.abs()}</td> },
		html! { <td class="text-center">{passive}</td> },
	];
	if let Some(idx) = ability_name_col {
		table_data.insert(
			*idx,
			html! {
				<td class="text-center" style="font-size: 12px; vertical-align: middle;">
					{skill.ability().abbreviated_name().to_uppercase()}
				</td>
			},
		);
	}

	let onclick = modal_dispatcher.callback({
		let skill = *skill;
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<SkillModal {skill} />},
				..Default::default()
			})
		}
	});

	html! {<tr {onclick}>{table_data}</tr>}
}

#[derive(Clone, PartialEq, Properties)]
struct SkillModalProps {
	pub skill: Skill,
}

#[function_component]
fn SkillModal(SkillModalProps { skill }: &SkillModalProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	let proficiency = state.skills().proficiency(*skill);
	let bonus = state.ability_modifier(skill.ability(), Some(*proficiency.value()));

	let prof_table = match proficiency.sources().is_empty() {
		true => html! {},
		false => html! {<div style="margin-bottom: 10px;">
		<table class="table table-compact table-striped m-0">
			<thead>
				<tr class="text-center" style="color: var(--bs-heading-color);">
					<th scope="col" style="width: 180px;">{"Proficiency"}</th>
					<th scope="col">{"Sources"}</th>
				</tr>
			</thead>
			<tbody>
				{proficiency.sources().iter().map(|(path, value)| {
					let source_text = crate::data::as_feature_path_text(&path).unwrap_or_default();
					html! {
						<tr>
							<td class="d-flex align-items-center">
								{*value}
								<span class="flex-grow-1 text-center" style="margin-left: 5px;">{value.as_display_name()}</span>
							</td>
							<td>{source_text}</td>
						</tr>
					}
				}).collect::<Vec<_>>()}
			</tbody>
		</table>
		</div>},
	};

	let roll_modifiers = {
		let mut entries = state
			.skills()
			.iter_skill_modifiers(*skill)
			.map(|(modifier, items)| items.iter().map(move |item| (modifier, item)))
			.flatten()
			.collect::<Vec<_>>();
		entries.sort_by_key(|(modifier, _)| *modifier);
		entries
	};

	let roll_modifiers_table = match roll_modifiers.is_empty() {
		true => html! {},
		false => html! {<div style="margin-bottom: 10px;">
			<table class="table table-compact table-striped m-0">
				<thead>
					<tr class="text-center" style="color: var(--bs-heading-color);">
						<th scope="col" style="width: 150px;">{"Modifier"}</th>
						<th scope="col">{"Target"}</th>
						<th scope="col">{"Source"}</th>
					</tr>
				</thead>
				<tbody>
					{roll_modifiers.into_iter().map(|(modifier, item)| html! {
						<tr>
							<td class="d-flex">
								<span aria-label={format!("{modifier:?}")} style="margin-left: 2px; display: block; height: 16px; width: 16px; vertical-align: middle; margin-top: -2px;">
									<ModifierIcon value={modifier} />
								</span>
								<span class="flex-grow-1 text-center" style="margin-left: 5px;">{modifier.display_name()}</span>
							</td>
							<td class="text-center">{item.context.clone().unwrap_or_else(|| "--".into())}</td>
							<td>{crate::data::as_feature_path_text(&item.source).unwrap_or_default()}</td>
						</tr>
					}).collect::<Vec<_>>()}
				</tbody>
			</table>
		</div>},
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">
				{skill.display_name()}
				<span style="margin-left: 10px;">{format!("({})", skill.ability().long_name())}</span>
			</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
				<span>{"Bonus:"}</span>
				<span style="margin-left: 5px;">{match bonus >= 0 { true => "+", false => "-", }}{bonus.abs()}</span>
			</div>
			{prof_table}
			{roll_modifiers_table}
			<div class="text-block">{skill.description()}</div>
		</div>
	</>}
}
