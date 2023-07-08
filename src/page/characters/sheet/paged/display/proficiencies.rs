use yew::prelude::*;
use crate::system::dnd5e::components::{Proficiencies, SpeedAndSenses, ProfBonus};

#[function_component]
pub fn Page() -> Html {
	html! {<>
		<div class="d-flex justify-content-center align-items-center">
			<ProfBonus />
			<SpeedAndSenses />
		</div>
		<Proficiencies />
	</>}
}