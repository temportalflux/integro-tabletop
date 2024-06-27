use crate::{
	bootstrap::components::Tooltip,
	components::{context_menu, mobile, AnnotatedNumber},
	page::characters::sheet::CharacterHandle,
	system::dnd5e::{
		components::glyph,
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
	let screen_size = mobile::use_mobile_kind();

	let ability_score = state.ability_scores().get(*ability);

	let onclick = context_menu::use_control_action({
		let ability = *ability;
		move |_, _context| context_menu::Action::open_root(ability.long_name(), html!(<Modal {ability} />))
	});

	let tooltip = (ability_score.iter_bonuses().count() > 0).then(|| {
		format!(
			"<div class=\"attributed-tooltip\">{}</div>",
			ability_score.iter_bonuses().fold(String::new(), |mut content, (bonus, path, included_in_total)| {
				if *included_in_total {
					let source_text = crate::data::as_feature_path_text(&path).unwrap_or_default();
					content += format!("<span>+{} ({source_text})</span>", bonus.value).as_str();
				}
				content
			})
		)
	});
	let score_modifier = html! {
		<AnnotatedNumber
			value={ability_score.score().modifier()}
			show_sign={true}
		/>
	};

	let roll_modifiers = state.skills()[*ability]
		.modifiers()
		.iter()
		.map(|(modifier, items)| {
			let tooltip = crate::data::as_feature_paths_html_custom(
				items.into_iter(),
				|(context, source)| (context.clone(), source.as_path()),
				|criteria, path_str| match criteria {
					Some(criteria) => format!("<div>{} ({})</div>", criteria, path_str),
					None => format!("<div>{}</div>", path_str),
				},
			);
			html!(<Tooltip tag={"div"} content={tooltip} use_html={true}>
			<span aria-label={format!("{modifier:?}")}>
				<glyph::RollModifier value={modifier} />
			</span>
		</Tooltip>)
		})
		.collect::<Vec<_>>();

	match screen_size {
		mobile::Kind::Desktop => html! {
			<div class="card ability-card m-1" style="border-color: var(--theme-frame-color-muted);">
				<div class="card-body text-center" {onclick}>
					<h6 class="card-title">{ability.long_name()}</h6>
					<div class="primary-stat">
						{score_modifier}
						{roll_modifiers}
					</div>
					<Tooltip classes={"secondary-stat"} content={tooltip} use_html={true}>
						{*ability_score.score()}
					</Tooltip>
				</div>
			</div>
		},
		mobile::Kind::Mobile => {
			let saving_throw_prof = state.saving_throws()[*ability].proficiencies();
			let mut saving_throw_modifier = state.ability_scores()[*ability].score().modifier();
			saving_throw_modifier += saving_throw_prof.value() * state.proficiency_bonus();

			html! {
				<div class="p-1 text-center" {onclick}>
					<div class="row" style="--bs-gutter-x: 0;">
						<div class="col">
							<h5>{ability.long_name()}</h5>
						</div>
						<div class="col-3">
							<h5>
								{match ability_score.score().modifier() >= 0 {
									true => "+",
									false => "-",
								}}
								{ability_score.score().modifier().abs()}
							</h5>
						</div>
					</div>
					<div class="row" style="--bs-gutter-x: 0;">
						<div class="col">
							<div style="font-size: 0.75rem;">{"Score"}</div>
						</div>
						<div class="col-3">
							<div>{*ability_score.score()}</div>
						</div>
					</div>
					<div class="row align-items-center" style="--bs-gutter-x: 0;">
						<div class="col-auto" style="font-size: 0.75rem;">
							<glyph::ProficiencyLevel value={saving_throw_prof.value()} />
						</div>
						<div class="col">
							<div style="font-size: 0.75rem;">{"Saving Throw"}</div>
						</div>
						<div class="col-3">
							{match saving_throw_modifier >= 0 {
								true => "+",
								false => "-",
							}}
							{saving_throw_modifier.abs()}
						</div>
					</div>
				</div>
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct AbilityProps {
	pub ability: Ability,
}

#[function_component]
pub fn AbilityModifiers() -> Html {
	let style = "height: 14px; margin-right: 2px; margin-top: -2px; width: 14px; vertical-align: middle;";
	let state = use_context::<CharacterHandle>().unwrap();
	let mut modifiers = Vec::new();
	for ability in EnumSet::<Ability>::all() {
		for (modifier, context, _source) in state.skills()[ability].modifiers().iter_all() {
			modifiers.push(html! {<div>
				<span class="d-inline-flex" aria-label="Advantage" {style}>
					<glyph::RollModifier value={modifier} />
				</span>
				<span>{"on "}{ability.abbreviated_name().to_uppercase()}{" checks"}</span>
				<span>
					{context.as_ref().map(|target| format!(" when {target}")).unwrap_or_default()}
				</span>
			</div>});
		}
	}
	let content = match modifiers.is_empty() {
		false => html! {<>{modifiers}</>},
		true => html!("None"),
	};
	html! {
		<div style="font-size: 11px;">
			{content}
		</div>
	}
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
	let skills = EnumSet::<Skill>::all().into_iter().filter(|skill| skill.ability() == *ability);
	html! {<>
		<h1>
			<glyph::Ability value={*ability} />
			<span class="ms-2">{ability.long_name()}</span>
		</h1>

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
	</>}
}
