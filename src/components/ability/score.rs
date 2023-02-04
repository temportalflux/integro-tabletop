use crate::components::AnnotatedNumber;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ScoreProps {
	pub title: String,
	pub score: i32,
}

#[function_component]
pub fn Score(ScoreProps { title, score }: &ScoreProps) -> Html {
	let modifier = (((score - 10) as f32) / 2f32).floor() as i32;
	html! {
		<div class="card ability-card" style="margin: 10px 5px; border-color: var(--theme-frame-color-muted);">
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
		<div class="row m-0" style="--bs-gutter-x: 0;">
			<div class="col">
				<Score title={"Strength"} score={9} />
				<Score title={"Dexterity"} score={11} />
				<Score title={"Constitution"} score={17} />
			</div>
			<div class="col">
				<Score title={"Intelligence"} score={18} />
				<Score title={"Wisdom"} score={14} />
				<Score title={"Charisma"} score={17} />
			</div>
		</div>
	}
}
