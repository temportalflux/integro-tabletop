use super::InventoryItemProps;
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

#[derive(Clone, PartialEq, Properties)]
pub struct ItemRowProps {
	pub id_path: Vec<Uuid>,
	pub item: Item,
	pub is_equipped: Option<bool>,
}

#[function_component]
pub fn ItemRow(
	ItemRowProps {
		id_path,
		item,
		is_equipped,
	}: &ItemRowProps,
) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_modal = modal_dispatcher.callback({
		let id_path = id_path.clone();
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("item"),
				content: html! {<ItemModal id_path={id_path.clone()} />},
				..Default::default()
			})
		}
	});

	html! {
		<tr class="align-middle" onclick={open_modal}>
			{is_equipped.as_ref().map(|is_equipped| html! {
				<td class="text-center">
					<ItemRowEquipBox
						id={id_path.last().unwrap().clone()}
						is_equipable={item.is_equipable()}
						can_be_equipped={item.can_be_equipped(&*state)}
						is_equipped={*is_equipped}
					/>
				</td>
			}).unwrap_or_default()}
			<td>{item.name.clone()}</td>
			<td class="text-center">{item.weight * item.quantity() as f32}{" lb."}</td>
			<td class="text-center">{item.quantity()}</td>
			<td style="width: 250px;">{item.notes.clone()}</td>
		</tr>
	}
}

#[function_component]
pub fn ItemModal(InventoryItemProps { id_path }: &InventoryItemProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let item = {
		let mut iter = id_path.iter();
		let mut item = None;
		while let Some(id) = iter.next() {
			item = match item {
				None => state.inventory().get_item(id),
				Some(prev_item) => match &prev_item.items {
					None => {
						return Html::default();
					}
					Some(container) => container.get_item(id),
				},
			};
		}
		item
	};
	let Some(item) = item else { return Html::default(); };
	// TODO: edit capability for properties:
	// name, notes, quantity(✔)
	// dndbeyond also supports worth and weight overrides, idk if I want that or not
	// TODO: buttons for:
	// (un)equip(✔), sell(?), (un)attune, move (between containers)
	// TODO: button to convert into custom item, which enables full control over all properties.
	// 		Or maybe this just uses some `inheiret` property and allows user to
	// 		override any property after copying from some source id.

	let on_delete = state.new_dispatch({
		let id_path = id_path.clone();
		let close_modal = modal_dispatcher.callback(|_| modal::Action::Close);
		move |_: MouseEvent, persistent, _| {
			let equipped = id_path.len() == 1 && persistent.inventory.is_equipped(&id_path[0]);
			let _item = persistent.inventory.remove_at_path(&id_path);
			close_modal.emit(());
			equipped.then_some(ActionEffect::Recompile)
		}
	});
	let mut item_props = ItemBodyProps::default();
	match &item.kind {
		ItemKind::Simple { .. } => {
			item_props.on_quantity_changed = Some(state.new_dispatch({
				let id_path = id_path.clone();
				move |amt, persistent, _| {
					if let Some(item) = persistent.inventory.get_mut_at_path(&id_path) {
						if let ItemKind::Simple { count } = &mut item.kind {
							*count = amt;
						}
					}
					None
				}
			}));
		}
		ItemKind::Equipment(_equipment) => {
			if id_path.len() == 1 {
				item_props.is_equipped = state.inventory().is_equipped(&id_path[0]);
				item_props.set_equipped = Some(state.new_dispatch({
					let id: Uuid = id_path[0].clone();
					move |should_be_equipped, persistent, _| {
						persistent.inventory.set_equipped(&id, should_be_equipped);
						Some(ActionEffect::Recompile)
					}
				}));
			}
		}
	}

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{item.name.clone()}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{item_body(item, &state, Some(item_props))}
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
