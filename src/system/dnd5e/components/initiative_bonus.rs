use crate::{
	components::{context_menu, AnnotatedNumber, AnnotatedNumberCard},
	page::characters::sheet::CharacterHandle,
	system::dnd5e::data::Ability,
};
use yew::prelude::*;

static TEXT: &'static str = "\
Initiative determines the order of turns during combat. When combat starts, every participant makes a \
Dexterity check to determine their place in the initiative order. The DM makes one roll for an entire \
group of identical creatures, so each member of the group acts at the same time.

The DM ranks the combatants in order from the one with the highest Dexterity check total to the one with the lowest. \
This is the order (called the initiative order) in which they act during each round. \
The initiative order remains the same from round to round.

If a tie occurs, the DM decides the order among tied DM-controlled creatures, and the players decide the order \
among their tied characters. The DM can decide the order if the tie is between a monster and a player character. \
Optionally, the DM can have the tied characters and monsters each roll \
a d20 to determine the order, highest roll going first.";

#[function_component]
pub fn InitiativeBonus() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let value = state.initiative_bonus();
	let on_click = context_menu::use_control_action({
		move |_| {
			context_menu::Action::open_root(
				format!("Initiative Bonus"),
				html! {<>
					<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
						<span>{Ability::Dexterity.long_name()}{":"}</span>
						<span style="margin-left: 5px;">{match value >= 0 { true => "+", false => "-", }}{value.abs()}</span>
					</div>
					<div class="text-block">
						{TEXT}
					</div>
				</>},
			)
		}
	});
	html! {
		<AnnotatedNumberCard header={"Initiative"} footer={"Bonus"} {on_click}>
			<AnnotatedNumber value={value} show_sign={true} />
		</AnnotatedNumberCard>
	}
}
