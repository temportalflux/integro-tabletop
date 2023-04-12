use crate::{
	components::modal,
	system::dnd5e::{
		components::{
			panel::{inventory::equip_toggle::ItemRowEquipBox, item_body, ItemBodyProps},
			SharedCharacter,
		},
		data::{
			character::ActionEffect,
			item::{Item, ItemKind},
		},
	},
};
use uuid::Uuid;
use yew::prelude::*;

use super::InventoryItemProps;

#[derive(Clone, PartialEq, Properties)]
pub struct ItemRowProps {
	pub id: Uuid,
	pub item: Item,
	pub is_equipped: bool,
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
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_modal = modal_dispatcher.callback({
		let id = id.clone();
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("item"),
				content: html! {<ItemModal id={id} />},
				..Default::default()
			})
		}
	});

	html! {
		<tr class="align-middle" onclick={open_modal}>
			<td class="text-center">
				<ItemRowEquipBox
					id={id.clone()}
					is_equipable={item.is_equipable()}
					can_be_equipped={item.can_be_equipped(&*state)}
					is_equipped={*is_equipped}
				/>
			</td>
			<td>{item.name.clone()}</td>
			<td class="text-center">{item.weight * item.quantity() as f32}{" lb."}</td>
			<td class="text-center">{item.quantity()}</td>
			<td style="width: 250px;">{item.notes.clone()}</td>
		</tr>
	}
}

#[function_component]
fn ItemModal(InventoryItemProps { id }: &InventoryItemProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let Some(item) = state.inventory().get_item(id) else { return html! {}; };
	let _is_equipped = state.inventory().is_equipped(id);
	// TODO: edit capability for properties:
	// name, notes, quantity(✔)
	// dndbeyond also supports worth and weight overrides, idk if I want that or not
	// TODO: buttons for:
	// (un)equip, sell(?), (un)attune, move (between containers)
	// TODO: button to convert into custom item, which enables full control over all properties.
	// 		Or maybe this just uses some `inheiret` property and allows user to
	// 		override any property after copying from some source id.

	let on_delete = state.new_dispatch({
		let id = id.clone();
		let close_modal = modal_dispatcher.callback(|_| modal::Action::Close);
		move |_: MouseEvent, persistent, _| {
			let equipped = persistent.inventory.is_equipped(&id);
			let _item = persistent.inventory.remove(&id);
			close_modal.emit(());
			equipped.then_some(ActionEffect::Recompile)
		}
	});
	let on_quantity_changed = state.new_dispatch({
		let id = id.clone();
		move |amt, persistent, _| {
			if let Some(item) = persistent.inventory.get_mut(&id) {
				if let ItemKind::Simple { count } = &mut item.kind {
					*count = amt;
				}
			}
			None
		}
	});

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{item.name.clone()}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{item_body(item, Some(ItemBodyProps {
				on_quantity_changed: Some(on_quantity_changed),
			}))}
			<span class="hr my-2" />
			<div class="d-flex justify-content-center">
				<button type="button" class="btn btn-sm btn-outline-theme" onclick={on_delete}>
					<i class="bi bi-trash me-1" />
					{"Delete"}
				</button>
			</div>
		</div>
	</>}
}
