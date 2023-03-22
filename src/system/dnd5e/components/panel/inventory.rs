use crate::{
	bootstrap::components::Tooltip,
	system::dnd5e::{
		components::{SharedCharacter, WalletInline},
		data::{character::ActionEffect, item::Item},
	},
};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn Inventory() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	html! {<>
		<div class="d-flex">
			<h5 class="my-auto">{"Equipment"}</h5>
			<WalletInline />
		</div>
		<span class="hr" />
		<table class="table table-compact m-0">
			<thead>
				<tr class="text-center" style="font-size: 0.7rem;">
					<th scope="col">{"Equip"}</th>
					<th scope="col">{"Name"}</th>
					<th scope="col">{"Weight"}</th>
					<th scope="col">{"Qty"}</th>
					<th scope="col">{"Notes"}</th>
				</tr>
			</thead>
			<tbody>
				{state.inventory().items_by_name().map(|entry| html! {
					<ItemRow id={entry.id.clone()} item={entry.item.clone()} is_equipped={entry.is_equipped} />
				}).collect::<Vec<_>>()}
			</tbody>
		</table>

	</>}
}

#[derive(Clone, PartialEq, Properties)]
pub struct ItemRowProps {
	id: Uuid,
	item: Item,
	is_equipped: bool,
}

#[function_component]
pub fn ItemRow(
	ItemRowProps {
		id,
		item,
		is_equipped,
	}: &ItemRowProps,
) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let on_click_row = Callback::from(|_| log::debug!("TODO: open item interface modal"));
	html! {
		<tr class="align-middle" onclick={on_click_row}>
			<td class="text-center">
				<ItemRowEquipBox
					id={id.clone()}
					is_equipable={item.is_equipable()}
					can_be_equipped={item.can_be_equipped(&*state)}
					is_equipped={*is_equipped}
				/>
			</td>
			<td>{item.name.clone()}</td>
			<td class="text-center">{item.weight}{" lb."}</td>
			<td class="text-center">{item.quantity()}</td>
			<td style="width: 250px;">{item.notes.clone()}</td>
		</tr>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct EquipBoxProps {
	id: Uuid,
	is_equipable: bool,
	can_be_equipped: Result<(), String>,
	is_equipped: bool,
}
#[function_component]
fn ItemRowEquipBox(
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
