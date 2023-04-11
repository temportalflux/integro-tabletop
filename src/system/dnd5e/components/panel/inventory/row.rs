use crate::{
	components::modal,
	system::dnd5e::{
		components::{panel::{inventory::equip_toggle::ItemRowEquipBox, item_body}, SharedCharacter},
		data::item::Item,
	},
};
use uuid::Uuid;
use yew::prelude::*;

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
			<td class="text-center">{item.weight}{" lb."}</td>
			<td class="text-center">{item.quantity()}</td>
			<td style="width: 250px;">{item.notes.clone()}</td>
		</tr>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ItemModalProps {
	id: Uuid,
}

#[function_component]
fn ItemModal(ItemModalProps { id }: &ItemModalProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let Some(item) = state.inventory().get_item(id) else { return html! {}; };
	let _is_equipped = state.inventory().is_equipped(id);
	// TODO: edit capability for properties:
	// name, notes, quantity
	// TODO: buttons for:
	// (un)equip, sell(?), (un)attune, move (between containers)
	// TODO: button to convert into custom item, which enables full control over all properties

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{item.name.clone()}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{item_body(item)}
		</div>
	</>}
}
