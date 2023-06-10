use super::InventoryItemProps;
use crate::{
	components::modal,
	page::characters::sheet::MutatorImpact,
	system::dnd5e::{
		components::{
			panel::{
				inventory::equip_toggle::ItemRowEquipBox, item_body, AddItemButton,
				AddItemOperation, ItemBodyProps,
			},
			CharacterHandle,
		},
		data::item::{Item, ItemKind},
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
	let state = use_context::<CharacterHandle>().unwrap();
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
	let state = use_context::<CharacterHandle>().unwrap();
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
		move |_: MouseEvent, persistent| {
			let equipped = id_path.len() == 1 && persistent.inventory.is_equipped(&id_path[0]);
			let _item = persistent.inventory.remove_at_path(&id_path);
			close_modal.emit(());
			match equipped {
				true => MutatorImpact::Recompile,
				false => MutatorImpact::None,
			}
		}
	});
	let mut item_props = ItemBodyProps::default();
	match &item.kind {
		ItemKind::Simple { .. } => {
			item_props.on_quantity_changed = Some(state.new_dispatch({
				let id_path = id_path.clone();
				move |amt, persistent| {
					if let Some(item) = persistent.inventory.get_mut_at_path(&id_path) {
						if let ItemKind::Simple { count } = &mut item.kind {
							*count = amt;
						}
					}
					MutatorImpact::None
				}
			}));
		}
		ItemKind::Equipment(_equipment) => {
			if id_path.len() == 1 {
				item_props.is_equipped = state.inventory().is_equipped(&id_path[0]);
				item_props.set_equipped = Some(state.new_dispatch({
					let id: Uuid = id_path[0].clone();
					move |should_be_equipped, persistent| {
						persistent.inventory.set_equipped(&id, should_be_equipped);
						MutatorImpact::Recompile
					}
				}));
			}
		}
	}

	// TODO: In order to move only part of a stack, we should have a form field to allow the user to split the itemstack
	// (taking stack - newsize and inserting that as a new item), so the user can move this stack to a new container.
	let move_button = html! {
		<AddItemButton
			btn_classes={classes!("btn-outline-theme", "btn-sm", "mx-1")}
			operation={AddItemOperation::Move {
				item_id: id_path.clone(),
				source_container: match id_path.len() {
					1 => None,
					// TODO: This should take the first n-1 elements as the parent path,
					// right now it assumes that the current item is always in a top-level container.
					n => Some(id_path[0..(n-1)].to_vec()),
				},
			}}
			on_click={state.new_dispatch({
				let close_modal = modal_dispatcher.callback(|_| modal::Action::Close);
				let id_path = id_path.clone();
				move |dst_id: Option<Vec<Uuid>>, persistent| {
					let Some(item) = persistent.inventory.remove_at_path(&id_path) else { return MutatorImpact::None; };
					persistent.inventory.insert_to(item, &dst_id);
					close_modal.emit(());
					MutatorImpact::None
				}
			})}
		/>
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{item.name.clone()}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body d-flex flex-column" style="min-height: 200px;">
			{item_body(item, &state, Some(item_props))}
			<span class="hr my-2" />
			<div class="d-flex justify-content-center mt-auto">
				{move_button}
				<button type="button" class="btn btn-sm btn-outline-theme mx-1" onclick={on_delete}>
					<i class="bi bi-trash me-1" />
					{"Delete"}
				</button>
			</div>
		</div>
	</>}
}
