use crate::{
	bootstrap::components::Tooltip,
	components::stop_propagation,
	page::characters::sheet::MutatorImpact,
	system::dnd5e::{components::CharacterHandle, data::character::Persistent},
	utility::InputExt,
};
use uuid::Uuid;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct EquipBoxProps {
	pub id: Uuid,
	pub is_equipable: bool,
	pub can_be_equipped: Result<(), String>,
	pub is_equipped: bool,
}

#[function_component]
pub fn ItemRowEquipBox(
	EquipBoxProps {
		id,
		is_equipable,
		can_be_equipped,
		is_equipped,
	}: &EquipBoxProps,
) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	if !*is_equipable {
		return html! { {"--"} };
	}

	let on_change = Callback::from({
		let id = id.clone();
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(should_be_equipped) = evt.input_checked() else { return; };
			state.dispatch(Box::new(move |persistent: &mut Persistent| {
				persistent.inventory.set_equipped(&id, should_be_equipped);
				MutatorImpact::Recompile
			}));
		}
	});

	html! {
		<Tooltip content={match *is_equipped {
			true => None,
			false => can_be_equipped.clone().err(),
		}}>
			<input
				class={"form-check-input equip"} type={"checkbox"}
				checked={*is_equipped}
				disabled={!*is_equipped && can_be_equipped.is_err()}
				onclick={stop_propagation()}
				onchange={on_change}
			/>
		</Tooltip>
	}
}
