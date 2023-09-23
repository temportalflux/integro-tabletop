use std::collections::HashMap;

use crate::{
	components::{context_menu, database::use_typed_fetch_callback, Spinner},
	page::characters::sheet::CharacterHandle,
	system::{
		core::SourceId,
		dnd5e::{
			components::{GeneralProp, WalletInline, WalletInlineButton},
			data::{
				character::{IndirectItem, StartingEquipment},
				item::{container::item::AsItem, Item},
			},
		},
	},
};
use uuid::Uuid;
use yew::prelude::*;

mod add_btn;
pub use add_btn::*;
mod browse_content;
pub use browse_content::*;
mod equip_toggle;
pub use equip_toggle::*;
mod item_content;
pub use item_content::*;
mod row;
pub use row::*;
use yew_hooks::use_is_first_mount;

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
	let state = use_context::<CharacterHandle>().unwrap();
	let open_browser = context_menu::use_control_action({
		|_, context| context_menu::Action::open_root("Item Browser", html!(<BrowseModal />))
	});
	let open_starting_equipment = context_menu::use_control_action({
		|_, context| {
			context_menu::Action::open_root(
				"Starting Equipment",
				html!(<BrowseStartingEquipment />),
			)
		}
	});

	let search_header = html! {
		<div class="input-group mt-2">
			<span class="input-group-text"><i class="bi bi-search"/></span>
			<input
				type="text" class="form-control"
				placeholder="Search item names, types, rarities, or tags"
			/>
			<button type="button" class="btn btn-outline-theme" onclick={open_browser}>{"Browse Items"}</button>
		</div>
	};

	if state.inventory().is_empty() {
		return html! {
			<div class="panel inventory">
				{search_header}
				<div class="empty-prompt">
					<div class="text">{"Your inventory is empty. Do you want to select starting equipment?"}</div>
					<button type="button" class="btn btn-sm btn-theme mt-2" onclick={open_starting_equipment}>{"Add Starting Equipment"}</button>
				</div>
			</div>
		};
	}

	// TODO: Implement search-inventory functionality
	// TODO: tag buttons to browse item containers
	let containers = state
		.inventory()
		.iter_by_name()
		.filter(|(_, entry)| entry.as_item().items.is_some())
		.map(|(id, _)| html! { <ContainerSection container_id={id.clone()} /> })
		.collect::<Vec<_>>();
	html! {
		<div class="panel inventory">
			{search_header}
			<div class="sections">
				<ContainerSection container_id={None} />
				{containers}
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ContainerSectionProps {
	container_id: Option<Uuid>,
}
#[function_component]
fn ContainerSection(ContainerSectionProps { container_id }: &ContainerSectionProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let context_menu = use_context::<context_menu::Control>().unwrap();

	let title: AttrValue;
	let wallet: Option<Html>;
	let rows: Vec<Html>;
	let can_equip_from = container_id.is_none();
	let open_modal: Option<Callback<MouseEvent>>;
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
			open_modal = None;
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
			open_modal = Some(Callback::from({
				let context_menu = context_menu.clone();
				let id_path = vec![container_id.clone()];
				let name = AttrValue::from(item.name.clone());
				move |_| {
					context_menu.dispatch(context_menu::Action::open_root(
						name.clone(),
						html!(<ItemModal id_path={id_path.clone()} />),
					))
				}
			}));
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

#[function_component]
fn BrowseStartingEquipment() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();

	// TODO: If the player's persistent inventory is empty,
	// show an option to add items based on their StartingEquipment.
	log::debug!(target: "inventory", "{:?}", state.starting_equipment());

	let mut sections = Vec::with_capacity(state.starting_equipment().len());
	for (options, source) in state.starting_equipment() {
		sections.push(html! {
			<div class="section-group">
				<h4 class="title">
					{crate::data::as_feature_path_text(source)}
				</h4>
				{options.iter().map(|group| html!(<Section kind={group.clone()} />)).collect::<Vec<_>>()}
			</div>
		});
	}

	// TODO: Needs a button to confirm, apply items, and close panel

	html! {
		<div class="starting-equipment">
			{sections}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SectionProps {
	kind: StartingEquipment,
	#[prop_or_default]
	prefix: Html,
	#[prop_or_default]
	disabled: bool,
}

#[function_component]
fn Section(SectionProps { kind, prefix, disabled }: &SectionProps) -> Html {
	// TODO: propogate disabled status for when an option in a group is not active
	let content = match kind {
		StartingEquipment::Currency(wallet) => html! {<>
			<div class="label">
				{prefix.clone()}
				<span>{"Get Currency"}</span>
			</div>
			<div class="wallet">
				<WalletInline wallet={*wallet} />
			</div>
		</>},
		StartingEquipment::IndirectItem(IndirectItem::Specific(id, quantity)) => {
			html!(<ItemById
				id={id.minimal().into_owned()} quantity={*quantity}
				prefix={prefix.clone()} disabled={*disabled}
			/>)
		},
		StartingEquipment::IndirectItem(IndirectItem::Custom(item)) => {
			html!(<SpecificItem
				item={std::rc::Rc::new(item.clone())} quantity={1}
				prefix={prefix.clone()} disabled={*disabled}
			/>)
		},
		StartingEquipment::Group { entries, pick } => {
			// TODO: checkbox next to each section (disabled if maxed, autoclear old if pick is 1)
			// TODO: Pick should always be a number, never none
			let checkbox = html!(<input type="checkbox" class="form-check-input slot missing-value" />);
			html! {<>
				<div class="label">
					{prefix.clone()}
					<span>{format!("Pick {} of", pick.unwrap_or(1))}</span>
				</div>
				<div class="group">
					{entries.iter().map(|group_option| {
						html!(<Section kind={group_option.clone()} prefix={checkbox.clone()} disabled={true} />)
					}).collect::<Vec<_>>()}
				</div>
			</>}
		}
		StartingEquipment::SelectItem(filter) => {
			// TODO: dropdown to show all items which match the filter
			// TODO: display item card when any are selected
			html! {<>
				<div class="label">
					{prefix.clone()}
					<span>{"Select an item"}</span>
				</div>
				<div class="select-item">
					<select class="form-select missing-value" disabled={*disabled}>
						<option selected={true}>{"Select Item..."}</option>
						<option value="id1">{"Item 1"}</option>
						<option value="id2">{"Item 2"}</option>
						<option value="id3">{"Item 3"}</option>
					</select>
				</div>
			</>}
		}
	};
	html!(<div class="section">{content}</div>)
}

#[derive(Clone, PartialEq, Properties)]
struct ItemByIdProps {
	id: SourceId,
	quantity: usize,
	#[prop_or_default]
	prefix: Html,
	#[prop_or_default]
	disabled: bool,
}
#[function_component]
fn ItemById(ItemByIdProps { id, quantity, prefix, disabled }: &ItemByIdProps) -> Html {
	let found_item = use_state(|| None::<std::rc::Rc<Item>>);
	let fetch_item = use_typed_fetch_callback(
		"Fetch Item".into(),
		Callback::from({
			let found_item = found_item.clone();
			move |item: Item| {
				found_item.set(Some(std::rc::Rc::new(item)));
			}
		}),
	);
	if use_is_first_mount() {
		fetch_item.emit(id.clone());
	}
	let Some(item) = &*found_item else {
		return html!(<Spinner />);
	};
	html!(<SpecificItem item={item.clone()} quantity={*quantity} prefix={prefix.clone()} disabled={*disabled} />)
}

#[derive(Clone, PartialEq, Properties)]
struct SpecificItemProps {
	item: std::rc::Rc<Item>,
	quantity: usize,
	#[prop_or_default]
	prefix: Html,
	#[prop_or_default]
	disabled: bool,
}
#[function_component]
fn SpecificItem(SpecificItemProps { item, quantity, prefix, disabled }: &SpecificItemProps) -> Html {
	html! {<>
		<div class="label">
			{prefix.clone()}
			<span>
				{"Get Item"}
				{(*quantity > 1).then(|| html!(format!(" (x{quantity})"))).unwrap_or_default()}
			</span>
		</div>
		<div class="specific-item">
			<ItemCard item={item.clone()} disabled={*disabled} />
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct ItemCardProps {
	item: std::rc::Rc<Item>,
	#[prop_or_default]
	disabled: bool,
}

#[function_component]
fn ItemCard(ItemCardProps { item, disabled }: &ItemCardProps) -> Html {
	let onclick = context_menu::use_control_action({
		let item = item.clone();
		move |_, context| context_menu::Action::open(
			&context,
			item.name.clone(),
			html!(<ItemInfo item={item.clone()} />),
		)
	});
	let mut classes = classes!("card", "item");
	if *disabled {
		classes.push("disabled");
	}

	html! {
		<div class={classes} {onclick}>
			<div class="card-body">
				<p class="card-title">{&item.name}</p>
				<button class="btn btn-theme btn-xs">
					<i class="bi bi-chevron-right" />
				</button>
			</div>
		</div>
	}
}
