use crate::{
	bootstrap::components::Tooltip,
	system::dnd5e::{components::SharedCharacter, data::character::ActionEffect},
};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
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
	let state = use_context::<SharedCharacter>().unwrap();
	if !*is_equipable {
		return html! { {"--"} };
	}

	let on_change = Callback::from({
		let id = id.clone();
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let should_be_equipped = input.checked();
			state.dispatch(Box::new(move |persistent, _| {
				persistent.inventory.set_equipped(&id, should_be_equipped);
				Some(ActionEffect::Recompile)
			}));
		}
	});

	html! {
		<Tooltip content={match *is_equipped {
			true => None,
			false => can_be_equipped.clone().err(),
		}}>
			<input
				class={"form-check-input"} type={"checkbox"}
				checked={*is_equipped}
				disabled={!*is_equipped && can_be_equipped.is_err()}
				onclick={Callback::from(|evt: web_sys::MouseEvent| evt.stop_propagation())}
				onchange={on_change}
			/>
		</Tooltip>
	}
}
