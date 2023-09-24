use crate::{
	components::{
		context_menu,
		database::{use_query_all_typed, use_typed_fetch_callback, QueryAllArgs, QueryStatus},
		Spinner, stop_propagation,
	},
	page::characters::sheet::{CharacterHandle, MutatorImpact},
	system::{
		core::{SourceId, System},
		dnd5e::{
			components::{WalletInline, WalletInlineButton},
			data::{
				character::{IndirectItem, StartingEquipment},
				item::{self, container::item::AsItem, Item},
			},
		},
	},
	utility::InputExt,
};
use std::{rc::Rc, path::{Path, PathBuf}};
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
		|_, _context| context_menu::Action::open_root("Item Browser", html!(<BrowseModal />))
	});
	let open_starting_equipment = context_menu::use_control_action({
		|_, _context| {
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
	let close_details = context_menu::use_close_fn::<()>();

	let mut missing_selections = false;
	let mut sections = Vec::with_capacity(state.starting_equipment().len());
	let selection_path = Path::new("starting_equipment").to_owned();
	for (idx, (options, source)) in state.starting_equipment().iter().enumerate() {
		let selection_path = selection_path.join(idx.to_string());
		let mut subsections = Vec::with_capacity(options.len());
		for (idx, element) in options.iter().enumerate() {
			let selection_path = selection_path.join(idx.to_string());
			if !missing_selections {
				// TODO: This doesnt account for groups which are not selected but contain a select-item, therefore always being missing
				for selection_key in element.get_required_selection_keys(&selection_path) {
					if state.get_first_selection(&selection_key).is_none() {
						missing_selections = true;
						break;
					}
				}
			}
			subsections.push(html!(<Section kind={element.clone()} {selection_path} />));
		}
		sections.push(html! {
			<div class="section-group">
				<h4 class="title">
					{crate::data::as_feature_path_text(source)}
				</h4>
				{subsections}
			</div>
		});
	}

	// TODO: Needs a button to confirm, apply items, and close panel
	let submit = Callback::from({
		let close_details = close_details.clone();
		move |_| {
			if missing_selections { return; };
			
			
			close_details.emit(());
		}
	});

	html! {
		<div class="starting-equipment">
			<div class="actions">
				<button type="button" onclick={submit} class={classes!(
					"btn", "btn-sm", "btn-success",
					missing_selections.then(|| "disabled")
				)}>
					{"Add Equipment & Close"}
				</button>
			</div>
			{sections}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SectionProps {
	selection_path: PathBuf,
	kind: StartingEquipment,
	#[prop_or_default]
	prefix: Html,
	#[prop_or_default]
	disabled: bool,
}

#[function_component]
fn Section(
	SectionProps {
		selection_path,
		kind,
		prefix,
		disabled,
	}: &SectionProps,
) -> Html {
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
		}
		StartingEquipment::IndirectItem(IndirectItem::Custom(item)) => {
			html!(<SpecificItem
				item={std::rc::Rc::new(item.clone())} quantity={1}
				prefix={prefix.clone()} disabled={*disabled}
			/>)
		}
		// TODO: Pick should always be a number, never none
		StartingEquipment::Group { entries, pick } => html!(
			<Group
				entries={entries.clone()} pick_max={pick.unwrap_or(1)}
				selection_path={selection_path.clone()}
				prefix={prefix.clone()} disabled={*disabled}
			/>
		),
		StartingEquipment::SelectItem(filter) => html!(
			<SelectItem
				filter={filter.clone()}
				selection_path={selection_path.clone()}
				prefix={prefix.clone()} disabled={*disabled}
			/>
		),
	};
	html!(<div class="section">{content}</div>)
}

#[derive(Clone, PartialEq, Properties)]
struct GroupProps {
	entries: Vec<StartingEquipment>,
	pick_max: usize,
	selection_path: PathBuf,
	#[prop_or_default]
	prefix: Html,
	#[prop_or_default]
	disabled: bool,
}
#[function_component]
fn Group(props: &GroupProps) -> Html {
	let GroupProps {
		entries,
		pick_max,
		selection_path,
		prefix,
		disabled,
	} = props;
	let state = use_context::<CharacterHandle>().unwrap();

	fn get_selected_indices(values: Option<&Vec<String>>) -> Vec<usize> {
		values.map(|values| {
			values.iter().filter_map(|s| s.parse::<usize>().ok()).collect::<Vec<_>>()
		}).unwrap_or_default()
	}
	let values = get_selected_indices(state.get_selections_at(&selection_path));
	let selected_options = use_state_eq(move || values);

	let set_option_selected = Callback::from({
		let state = state.clone();
		let selection_path = selection_path.clone();
		let pick_max = *pick_max;
		let selected_options = selected_options.clone();
		move |(idx, should_be_selected): (usize, bool)| {
			let key = selection_path.clone();
			let value = idx.to_string();
			let selected_options = selected_options.clone();
			state.dispatch(move |persistent| {
				match (should_be_selected, pick_max) {
					(false, _) => {
						persistent.remove_selected_value(&key, value);
					}
					(true, 1) => {
						persistent.set_selected_value(&key, value);
					}
					(true, _) => {
						persistent.insert_selection(&key, value);
					}
				}

				let values = get_selected_indices(persistent.get_selections_at(&key));
				selected_options.set(values);

				MutatorImpact::None
			});
		}
	});

	let can_select_new = *pick_max == 1 || selected_options.len() < *pick_max;

	html! {<>
		<div class="label">
			{prefix.clone()}
			<span>{format!("Pick {pick_max} of")}</span>
		</div>
		<div class="group">
			{entries.iter().enumerate().map(move |(idx, group_option)| {
				html!(<GroupOption
					group_option={group_option.clone()}

					{can_select_new}
					has_any_selected={!selected_options.is_empty()}
					is_selected={selected_options.contains(&idx)}
					set_option_selected={set_option_selected.reform(move |toggle| (idx, toggle))}

					selection_path={selection_path.join(idx.to_string())}
					disabled={*disabled}
				/>)
			}).collect::<Vec<_>>()}
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct GroupOptionProps {
	group_option: StartingEquipment,

	can_select_new: bool,
	has_any_selected: bool,
	is_selected: bool,
	set_option_selected: Callback<bool>,

	selection_path: PathBuf,
	#[prop_or_default]
	disabled: bool,
}
#[function_component]
fn GroupOption(props: &GroupOptionProps) -> Html {
	let GroupOptionProps {
		group_option,
		can_select_new, has_any_selected,
		is_selected, set_option_selected,
		selection_path, disabled
	} = props;

	let onchange = Callback::from({
		let set_option_selected = set_option_selected.clone();
		move |evt: web_sys::Event| {
			let Some(should_be_picked) = evt.input_checked() else { return; };
			set_option_selected.emit(should_be_picked);
		}
	});

	let checkbox = html!(<input
		type="checkbox"
		class={classes!(
			"form-check-input", "slot",
			(!disabled && !has_any_selected).then(|| "missing-value"),
		)}
		checked={*is_selected}
		disabled={*disabled || (!can_select_new && !is_selected)}
		onclick={stop_propagation()}
		{onchange}
	/>);
	html!(<Section
		kind={group_option.clone()}
		selection_path={selection_path.clone()}
		prefix={checkbox.clone()} disabled={*disabled || !is_selected}
	/>)
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
fn ItemById(
	ItemByIdProps {
		id,
		quantity,
		prefix,
		disabled,
	}: &ItemByIdProps,
) -> Html {
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
fn SpecificItem(
	SpecificItemProps {
		item,
		quantity,
		prefix,
		disabled,
	}: &SpecificItemProps,
) -> Html {
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
struct SelectItemProps {
	filter: item::Restriction,
	selection_path: PathBuf,
	#[prop_or_default]
	prefix: Html,
	#[prop_or_default]
	disabled: bool,
}
#[function_component]
fn SelectItem(
	SelectItemProps {
		filter,
		selection_path,
		prefix,
		disabled,
	}: &SelectItemProps,
) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let selected = state.get_first_selection(&selection_path).cloned();
	
	let query_handle = use_query_all_typed::<Item>(
		true,
		Some(QueryAllArgs {
			system: crate::system::dnd5e::DnD5e::id().into(),
			criteria: Some(filter.as_criteria().into()),
			..Default::default()
		}),
	);
	let onchange = Callback::from({
		let state = state.clone();
		let key = selection_path.clone();
		move |evt: web_sys::Event| {
			let Some(value) = evt.select_value() else { return; };
			let next_value = (!value.is_empty()).then(move || value);
			let key = key.clone();
			state.dispatch(move |persistent| {
				persistent.set_selected(key, next_value);
				MutatorImpact::None
			});
		}
	});

	let mut selected_item = None;
	match (&selected, query_handle.status()) {
		(Some(id_str), QueryStatus::Success(items)) => {
			for item in items {
				if item.id.to_string() == *id_str {
					selected_item = Some(Rc::new(item.clone()));
					break;
				}
			}
		}
		_ => {}
	};

	let empty_option = |text: &'static str| {
		html!(<option selected={selected.is_none()}>{text}</option>)
	};
	let options = match query_handle.status() {
		QueryStatus::Pending => empty_option("Pending..."),
		QueryStatus::Empty => empty_option("No Options"),
		QueryStatus::Failed(err) => {
			log::error!(target: "starting-equipment", "Failed to find items for {filter:?}: {err:?}");
			empty_option("Unavailable")
		}
		QueryStatus::Success(items) => {
			let mut options = vec![empty_option("Select Item...")];
			for item in items {
				let id_str = item.id.to_string();
				let is_selected = selected.as_ref() == Some(&id_str);
				options.push(html! {
					<option selected={is_selected} value={id_str}>
						{&item.name}
					</option>
				});
			}
			html!(<>{options}</>)
		}
	};
	let mut select_class = classes!("form-select");
	if selected.is_none() && !*disabled {
		select_class.push("missing-value");
	}
	html! {<>
		<div class="label">
			{prefix.clone()}
			<span>{"Select an item"}</span>
		</div>
		<div class="select-item">
			<div class="content">
				<select
					class={select_class}
					disabled={*disabled}
					{onchange}
				>
					{options}
				</select>
				{selected_item.map(|item| html! {
					<ItemCard {item} disabled={*disabled} />
				})}
			</div>
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
		move |_, context| {
			context_menu::Action::open(
				&context,
				item.name.clone(),
				html!(<ItemInfo item={item.clone()} />),
			)
		}
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
