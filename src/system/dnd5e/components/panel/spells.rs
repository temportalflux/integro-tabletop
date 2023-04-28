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
	let spell_slots = state.spellcasting().spell_slots(&*state);

	html! {
		<div style="overflow-y: scroll; height: 510px;">
			<div>
				{format!("Spell Slots: {:?}", spell_slots)}
			</div>
			<div>
				{state.spellcasting().iter_casters().map(|caster| {
					html! {
						<div>
							<strong>{caster.name().clone()}</strong>
							<div>
								{format!("Restriction: {:?}", caster.restriction)}
							</div>
							<div>
								{format!("Cantrip Capacity: {:?}", caster.cantrip_capacity(state.persistent()))}
							</div>
							{caster.cantrip_data_path().map(|key| {
								html! { <div>{format!("Cantrips: {:?}", state.get_selections_at(&key))}</div> }
							}).unwrap_or_default()}
							<div>{format!("Spell Capacity: {:?}", caster.spell_capacity(&state))}</div>
							<div>{format!("Max Level: {:?}", caster.max_spell_rank(&state))}</div>
							<div>{format!("Spells: {:?}", state.get_selections_at(&caster.spells_data_path()))}</div>
						</div>
					}
				}).collect::<Vec<_>>()}
			</div>
			<div>
				<strong>{"Always Prepared:"}</strong>
				{entries}
			</div>
			<div>
				{format!("{:?}", state.spellcasting())}
			</div>
		</div>
	}
}
