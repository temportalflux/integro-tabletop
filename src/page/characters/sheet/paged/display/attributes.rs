use crate::system::dnd5e::components::{
	ArmorClass, ConditionsCard, DefensesCard, InitiativeBonus, ProfBonus, Proficiencies, SpeedAndSenses,
};
use yew::prelude::*;

#[function_component]
pub fn Page() -> Html {
	html! {<>
		<div class="d-flex justify-content-center align-items-center">
			<ProfBonus />
			<InitiativeBonus />
			<ArmorClass />
		</div>
		<div class="d-flex justify-content-center align-items-center">
			<SpeedAndSenses />
		</div>
		<DefensesCard />
		<ConditionsCard />
		<Proficiencies />
	</>}
}
