use crate::{
	components::AnnotatedNumber,
	system::dnd5e::{character::AttributedValue, Ability},
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ScoreProps {
	pub ability: Ability,
	pub score: AttributedValue<i32>,
}

#[function_component]
pub fn Score(ScoreProps { ability, score }: &ScoreProps) -> Html {
	let modifier = (((score.value() - 10) as f32) / 2f32).floor() as i32;
	html! {
		<div class="card ability-card" style="margin: 10px 5px; border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center">
				<h6 class="card-title">{ability.long_name()}</h6>
				<div class="primary-stat">
					<AnnotatedNumber value={modifier} show_sign={true} />
				</div>
				<div class="secondary-stat">{score.value()}</div>
			</div>
		</div>
	}
}
