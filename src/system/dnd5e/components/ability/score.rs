use crate::{
	bootstrap::components::Tooltip,
	components::{modal, AnnotatedNumber},
	system::dnd5e::{
		components::{ability::AbilityGlyph, SharedCharacter},
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
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let (score, attributed_to) = state.ability_score(*ability);
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

	let tooltip = (!attributed_to.is_empty()).then(|| {
		format!(
			"<div class=\"attributed-tooltip\">{}</div>",
			attributed_to
				.iter()
				.fold(String::new(), |mut content, (path, value)| {
					let source_text = crate::data::as_feature_path_text(&path);
					let sign = source_text
						.is_some()
						.then(|| match *value >= 0 {
							true => "+",
							false => "-",
						})
						.unwrap_or_default();
					let path_name = source_text.unwrap_or("Base Score".into());
					let value = value.abs();
					content += format!("<span>{sign}{value} ({path_name})</span>").as_str();
					content
				})
		)
	});
	html! {
		<div class="card ability-card m-1" style="border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center" {onclick}>
				<h6 class="card-title">{ability.long_name()}</h6>
				<div class="primary-stat">
					<AnnotatedNumber value={score.modifier()} show_sign={true} />
				</div>
				<Tooltip classes={"secondary-stat"} content={tooltip} use_html={true}>{*score}</Tooltip>
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ModalProps {
	ability: Ability,
}
#[function_component]
fn Modal(ModalProps { ability }: &ModalProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let (score, attributed_to) = state.ability_score(*ability);
	let modifier = score.modifier();
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

			<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
				<span>{"Total Score:"}</span>
				<span style="margin-left: 5px;">{*score}</span>
			</div>
			<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
				<span>{"Modifier:"}</span>
				<span style="margin-left: 5px;">{match modifier >= 0 { true => "+", false => "-", }}{modifier.abs()}</span>
			</div>

			<table class="table table-compact table-striped m-0">
				<thead>
					<tr class="text-center" style="color: var(--bs-heading-color);">
						<th scope="col">{"Source"}</th>
						<th scope="col">{"Value"}</th>
					</tr>
				</thead>
				<tbody>
					{attributed_to.into_iter().map(|(path, value)| {
						let source_text = crate::data::as_feature_path_text(&path);
						let is_base = source_text.is_none();
						let source_text = source_text.unwrap_or("Base Score".into());
						let value_sign = match is_base {
							true => "",
							false => match value >= 0 { true => "+", false => "-", },
						};
						html! {
							<tr>
								<td>{source_text}</td>
								<td class="text-center">{value_sign}{value.abs()}</td>
							</tr>
						}
					}).collect::<Vec<_>>()}
				</tbody>
			</table>

			<div style="margin-top: 10px; margin-bottom: 10px;">
				{ability.short_description()}
			</div>

			<h6>{ability.long_name()}{" Checks"}</h6>
			<div style="white-space: pre-line; margin-bottom: 10px;">{ability.checks_description()}</div>

			{skills.map(|skill| html! {<>
				<h6>{skill.display_name()}</h6>
				<div style="white-space: pre-line; margin-bottom: 10px;">{skill.description()}</div>
			</>}).collect::<Vec<_>>()}

			{ability.addendum_description().into_iter().map(|(title, content)| html! {<>
				<h6>{title}</h6>
				<div style="white-space: pre-line; margin-bottom: 10px;">{content}</div>
			</>}).collect::<Vec<_>>()}

		</div>
	</>}
}
