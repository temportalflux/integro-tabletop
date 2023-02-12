use crate::{
	components::AnnotatedNumber,
	system::dnd5e::{character::{AttributedValue, State}, Ability}, data::ContextMut,
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ScoreProps {
	pub ability: Ability,
}

#[function_component]
pub fn Score(ScoreProps { ability }: &ScoreProps) -> Html {
	let state = use_context::<ContextMut<State>>().unwrap();
	let (score, _attributed_to) = state.ability_score(*ability);
	html! {
		<div class="card ability-card" style="margin: 10px 5px; border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center">
				<h6 class="card-title">{ability.long_name()}</h6>
				<div class="primary-stat">
					<AnnotatedNumber value={score.modifier()} show_sign={true} />
				</div>
				<div class="secondary-stat">{*score}</div>
			</div>
		</div>
	}
}
