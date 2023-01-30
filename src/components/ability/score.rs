use yew::prelude::*;
use crate::components::AnnotatedNumber;

#[derive(Clone, PartialEq, Properties)]
pub struct ScoreProps {
	pub title: String,
	pub score: i32,
}

#[function_component]
pub fn Score(ScoreProps { title, score }: &ScoreProps) -> Html {
	let modifier = (((score - 10) as f32) / 2f32).floor() as i32;
	html! {
		<div class="card ability-card" style="border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center">
				<h6 class="card-title">{title.clone()}</h6>
				<div class="primary-stat">
					<AnnotatedNumber value={modifier} show_sign={true} />
				</div>
				<div class="secondary-stat">{score}</div>
			</div>
		</div>
	}
}

#[function_component]
pub fn ScoreContainer() -> Html {
	html! {
		<div id="ability-container" class="card" style="width: 20em; border-color: var(--theme-frame-color);">
			<div class="card-body text-center">
				<h5 class="card-title">{"Ability Scores"}</h5>
				<div class="container overflow-hidden text-center">
					<div class="row gy-2" style="margin-top: 0;">
						<div class="col gx-2">
							<Score title={"Strength"} score={9} />
						</div>
						<div class="col gx-2">
							<Score title={"Dexterity"} score={11} />
						</div>
						<div class="col gx-2">
							<Score title={"Constitution"} score={17} />
						</div>
					</div>
					<div class="row gy-2" style="margin-top: 0;">
						<div class="col gx-2">
							<Score title={"Intelligence"} score={18} />
						</div>
						<div class="col gx-2">
							<Score title={"Wisdom"} score={14} />
						</div>
						<div class="col gx-2">
							<Score title={"Charisma"} score={17} />
						</div>
					</div>
				</div>
			</div>
		</div>
	}
}
