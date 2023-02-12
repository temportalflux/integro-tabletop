use crate::system::dnd5e::{Ability, ProficiencyLevel, Skill};
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
		let rows = skills
			.into_iter()
			.map(|skill| {
				// From character: the ability modifier for `skill.ability()`
				let ability_modifier = 1;
				// From character: the proficiency level for this skill
				let proficiency = match skill.ability() {
					Ability::Intelligence => ProficiencyLevel::Full,
					Ability::Dexterity => ProficiencyLevel::Double,
					Ability::Strength => ProficiencyLevel::Half,
					_ => ProficiencyLevel::None,
				};
				// From character: the proficiency bonus
				let prof_bonus = 2;
				let modifier = ability_modifier + (proficiency * prof_bonus);
				let passive = 10 + modifier;
				let mut table_data = vec![
					html! { <td class="text-center">{proficiency}</td> },
					html! { <td>{skill.display_name()}</td> },
					html! { <td class="text-center">{if modifier >= 0 { "+" } else { "-" }}{modifier.abs()}</td> },
					html! { <td class="text-center">{passive}</td> },
				];
				if let Some(idx) = &insert_ability_col_at {
					table_data.insert(
						*idx,
						html! {
							<td class="text-center" style="font-size: 12px; vertical-align: middle;">
								{skill.ability().abbreviated_name().to_uppercase()}
							</td>
						},
					);
				}
				html! {<tr>{table_data}</tr>}
			})
			.collect::<Vec<_>>();
		segments.push(html! {
			<table class="table table-compact m-0">
				<thead>
					<tr class="text-center" style="font-size: 0.7rem;">{headers.clone()}</tr>
				</thead>
				<tbody>{rows}</tbody>
			</table>
		});
	}

	let onclick = {
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
					<a class="dropdown-item" value={value.display_name()} onclick={onclick.clone()}>
						{value.icon_html()}
						<span style="margin-left: 5px;">{value.display_name()}</span>
					</a>
				}).collect::<Vec<_>>()}
			</div>
		</div>
		{segments}
	</>}
}
