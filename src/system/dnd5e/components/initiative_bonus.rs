use crate::{
	components::{modal, AnnotatedNumber, AnnotatedNumberCard},
	system::dnd5e::{components::SharedCharacter, data::Ability},
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
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let value = state.initiative_bonus();
	let on_click = modal_dispatcher.callback({
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<>
					<div class="modal-header">
						<h1 class="modal-title fs-4">{"Initiative Bonus"}</h1>
						<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
					</div>
					<div class="modal-body">
						<div class="text-center fs-5" style="width: 100%; margin-bottom: 10px;">
							<span>{Ability::Dexterity.long_name()}{":"}</span>
							<span style="margin-left: 5px;">{match value >= 0 { true => "+", false => "-", }}{value.abs()}</span>
						</div>
						<div class="text-block">
							{TEXT}
						</div>
					</div>
				</>},
				..Default::default()
			})
		}
	});
	html! {
		<AnnotatedNumberCard header={"Initiative"} footer={"Bonus"} {on_click}>
			<AnnotatedNumber value={value} show_sign={true} />
		</AnnotatedNumberCard>
	}
}
