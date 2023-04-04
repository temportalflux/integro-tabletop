use crate::{
	components::modal,
	system::dnd5e::components::{SharedCharacter, WalletInline},
};
use yew::prelude::*;

mod browse_content;
pub use browse_content::*;
mod equip_toggle;
pub use equip_toggle::*;
mod row;
pub use row::*;

#[function_component]
pub fn Inventory() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let open_browser = modal_dispatcher.callback(|_| {
		modal::Action::Open(modal::Props {
			centered: true,
			scrollable: true,
			root_classes: classes!("inventory"),
			content: html! {<BrowseModal />},
			..Default::default()
		})
	});

	// TODO: Implement search-inventory functionality
	// TODO: tag buttons to browse item containers
	html! {<>
		<div class="input-group mt-2">
			<span class="input-group-text"><i class="bi bi-search"/></span>
			<input
				type="text" class="form-control"
				placeholder="Search item names, types, rarities, or tags"
			/>
			<button type="button" class="btn btn-outline-theme" onclick={open_browser}>{"Browse Items"}</button>
		</div>
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
