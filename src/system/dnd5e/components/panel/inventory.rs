use crate::{
	components::modal,
	system::{
		core::SourceId,
		dnd5e::{
			components::{SharedCharacter, WalletInlineButton},
			data::item::AsItem,
		},
	},
};
use uuid::Uuid;
use yew::prelude::*;

mod browse_content;
pub use browse_content::*;
mod equip_toggle;
pub use equip_toggle::*;
mod item_content;
pub use item_content::*;
mod row;
pub use row::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SystemItemProps {
	pub id: SourceId,
}

#[derive(Clone, PartialEq, Properties)]
pub struct InventoryItemProps {
	pub id_path: Vec<uuid::Uuid>,
}

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
	let containers = state
		.inventory()
		.iter_by_name()
		.filter(|(_, entry)| entry.as_item().items.is_some())
		.map(|(id, _)| html! { <ContainerSection container_id={id.clone()} /> })
		.collect::<Vec<_>>();
	html! {<>
		<div class="input-group mt-2">
			<span class="input-group-text"><i class="bi bi-search"/></span>
			<input
				type="text" class="form-control"
				placeholder="Search item names, types, rarities, or tags"
			/>
			<button type="button" class="btn btn-outline-theme" onclick={open_browser}>{"Browse Items"}</button>
		</div>
		<ContainerSection container_id={None} />
		{containers}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct ContainerSectionProps {
	container_id: Option<Uuid>,
}
#[function_component]
fn ContainerSection(ContainerSectionProps { container_id }: &ContainerSectionProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let open_modal = match container_id {
		None => None,
		Some(id) => Some(modal_dispatcher.callback({
			let id_path = vec![id.clone()];
			move |_| {
				modal::Action::Open(modal::Props {
					centered: true,
					scrollable: true,
					root_classes: classes!("item"),
					content: html! {<ItemModal id_path={id_path.clone()} />},
					..Default::default()
				})
			}
		})),
	};

	let title: AttrValue;
	let wallet: Option<Html>;
	let rows: Vec<Html>;
	let can_equip_from = container_id.is_none();
	match container_id {
		None => {
			let container = state.inventory();
			title = "Equipment".into();
			wallet = Some(html! { <WalletInlineButton id={None} /> });
			rows = container
				.iter_by_name()
				.filter(|(_, entry)| entry.as_item().items.is_none())
				.map(|(id, entry)| {
					html! {
						<ItemRow id_path={vec![id.clone()]} item={entry.item.clone()} is_equipped={entry.is_equipped} />
					}
				})
				.collect::<Vec<_>>();
		}
		Some(container_id) => {
			let Some(item) = state.inventory().get_item(container_id) else { return Html::default(); };
			let Some(container) = &item.items else { return Html::default(); };
			title = item.name.clone().into();
			wallet = (!container.wallet().is_empty())
				.then(|| html! { <WalletInlineButton id={container_id.clone()} /> });
			rows = container
				.iter_by_name()
				.map(|(item_id, item)| {
					html! {
						<ItemRow id_path={vec![container_id.clone(), item_id.clone()]} item={item.clone()} />
					}
				})
				.collect::<Vec<_>>();
		}
	}
	html! {
		<div class="item-container mb-3">
			<div class="d-flex" onclick={open_modal}>
				<h5 class="ms-2 my-auto">{title}</h5>
				{wallet.unwrap_or_default()}
			</div>
			<span class="hr" />
			<table class="table table-compact m-0">
				<thead>
					<tr class="text-center" style="font-size: 0.7rem;">
						{can_equip_from.then(|| html! {
							<th scope="col">{"Equip"}</th>
						}).unwrap_or_default()}
						<th scope="col">{"Name"}</th>
						<th scope="col">{"Weight"}</th>
						<th scope="col">{"Qty"}</th>
						<th scope="col">{"Notes"}</th>
					</tr>
				</thead>
				<tbody>
					{rows}
				</tbody>
			</table>
		</div>
	}
}
