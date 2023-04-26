use crate::{
	components::modal,
	system::dnd5e::{components::SharedCharacter, DnD5e},
};
use yew::prelude::*;

#[function_component]
pub fn Spells() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let mut entries = Vec::new();
	for (spell_id, _) in state.spellcasting().prepared_spells() {
		entries.push(html! {<div>
			{spell_id.to_string()}
		</div>});
	}

	html! {<>
		{entries}
	</>}
}
