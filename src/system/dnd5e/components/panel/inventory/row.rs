use crate::system::dnd5e::{
	components::{panel::inventory::equip_toggle::ItemRowEquipBox, SharedCharacter},
	data::item::Item,
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
