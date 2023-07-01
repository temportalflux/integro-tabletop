use crate::{
	bootstrap::components::Tooltip,
	components::{modal, AnnotatedNumber},
	page::characters::sheet::CharacterHandle,
	system::dnd5e::{
		components::ability::AbilityGlyph,
		data::{Ability, Skill},
	},
};
use enumset::EnumSet;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ScoreProps {
	pub ability: Ability,
}

#[function_component]
pub fn Score(ScoreProps { ability }: &ScoreProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	// TODO: Display roll modifiers for ability checks.
	// Data is stored in `state.skills().iter_ability_modifiers()`

	let ability_score = state.ability_scores().get(*ability);
	let onclick = modal_dispatcher.callback({
		let ability = *ability;
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("ability-score"),
				content: html! {<Modal {ability} />},
				..Default::default()
			})
		}
	});

	let tooltip = (ability_score.iter_bonuses().count() > 0).then(|| {
		format!(
			"<div class=\"attributed-tooltip\">{}</div>",
			ability_score.iter_bonuses().fold(
				String::new(),
				|mut content, (bonus, path, included_in_total)| {
					if *included_in_total {
						let source_text =
							crate::data::as_feature_path_text(&path).unwrap_or_default();
						content +=
							format!("<span>+{} ({source_text})</span>", bonus.value).as_str();
					}
					content
				}
			)
		)
	});
	html! {
		<div class="card ability-card m-1" style="border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center" {onclick}>
				<h6 class="card-title">{ability.long_name()}</h6>
				<div class="primary-stat">
					<AnnotatedNumber value={ability_score.score().modifier()} show_sign={true} />
				</div>
				<Tooltip classes={"secondary-stat"} content={tooltip} use_html={true}>{*ability_score.score()}</Tooltip>
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct AbilityProps {
	pub ability: Ability,
}

#[function_component]
pub fn ScoreBreakdown(AbilityProps { ability }: &AbilityProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let ability_score = state.ability_scores().get(*ability);
	let modifier = ability_score.score().modifier();
	html! {<>
		<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
			<span>{"Total Score:"}</span>
			<span style="margin-left: 5px;">{*ability_score.score()}</span>
		</div>

		<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
			<span>{"Modifier:"}</span>
			<span style="margin-left: 5px;">{match modifier >= 0 { true => "+", false => "-", }}{modifier.abs()}</span>
		</div>

		<h6>{"Bonuses"}</h6>
		<table class="table table-compact table-striped m-0">
			<thead>
				<tr class="text-center" style="color: var(--bs-heading-color);">
					<th scope="col">{"Value"}</th>
					<th scope="col">{"Max Total"}</th>
					<th scope="col">{"Source"}</th>
				</tr>
			</thead>
			<tbody>
				{ability_score.iter_bonuses().map(|(bonus, path, was_included)| {
					html! {<tr>
						<td class="text-center">{bonus.value}</td>
						<td class="text-center">{match &bonus.max_total {
							None => html! {{"None" }},
							Some(max) => html! {<span>
								{max}
								{match was_included {
									true => "✅",
									false => "❌",
								}}
							</span>},
						}}</td>
						<td>{crate::data::as_feature_path_text(&path).unwrap_or_default()}</td>
					</tr>}
				}).collect::<Vec<_>>()}
			</tbody>
		</table>

		<h6>{"Maximum Value"}</h6>
		<table class="table table-compact table-striped m-0">
			<caption>{"The largest value of these is used as the maximum bound for the score above."}</caption>
			<thead>
				<tr class="text-center" style="color: var(--bs-heading-color);">
					<th scope="col">{"Value"}</th>
					<th scope="col">{"Source"}</th>
				</tr>
			</thead>
			<tbody>
				{ability_score.iter_max_increases().map(|(value, path)| {
					html! {<tr>
						<td class="text-center">{value}</td>
						<td>{crate::data::as_feature_path_text(&path).unwrap_or_default()}</td>
					</tr>}
				}).collect::<Vec<_>>()}
			</tbody>
		</table>
	</>}
}

#[function_component]
fn Modal(AbilityProps { ability }: &AbilityProps) -> Html {
	let skills = EnumSet::<Skill>::all()
		.into_iter()
		.filter(|skill| skill.ability() == *ability);
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">
				<AbilityGlyph ability={*ability} />
				{ability.long_name()}
			</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<ScoreBreakdown ability={*ability} />

			<div style="margin-top: 10px; margin-bottom: 10px;">
				{ability.short_description()}
			</div>

			<h6>{ability.long_name()}{" Checks"}</h6>
			<div class="text-block" style="margin-bottom: 10px;">{ability.checks_description()}</div>

			{skills.map(|skill| html! {<>
				<h6>{skill.display_name()}</h6>
				<div class="text-block" style="margin-bottom: 10px;">{skill.description()}</div>
			</>}).collect::<Vec<_>>()}

			{ability.addendum_description().into_iter().map(|(title, content)| html! {<>
				<h6>{title}</h6>
				<div class="text-block" style="margin-bottom: 10px;">{content}</div>
			</>}).collect::<Vec<_>>()}

		</div>
	</>}
}
