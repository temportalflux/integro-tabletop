use crate::{
	system::{
		core::{ModuleId, SourceId},
		dnd5e::{
			components::{
				editor::CollapsableCard,
				panel::{item_body, AddItemButton, AddItemOperation, SystemItemProps},
				validate_uint_only, SharedCharacter, WalletInline,
			},
			data::{
				character::Persistent,
				currency::Wallet,
				item::{Item, ItemKind},
			},
			DnD5e,
		},
	},
	utility::InputExt,
};
use uuid::Uuid;
use yew::prelude::*;
use yew_hooks::use_async;

fn now() -> f64 {
	let window = web_sys::window().expect("missing window");
	let perf = window.performance().expect("missing performance");
	perf.now()
}

#[function_component]
pub fn BrowseModal() -> Html {
	static DEFAULT_RESULT_LIMIT: usize = 10;
	static SEARCH_BUDGET_MS: u64 = 2;
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();

	let search_params = use_state(|| SearchParams::default());
	let result_limit = use_state_eq(|| Some(DEFAULT_RESULT_LIMIT));
	let search_results = use_async({
		let system = system.clone();
		let search_params = search_params.clone();
		let result_limit = result_limit.clone();
		async move {
			if search_params.is_empty() {
				return Ok((Vec::<SourceId>::new(), false));
			}

			let start = now();

			let mut stopped_early = false;
			let mut matched = Vec::with_capacity(result_limit.unwrap_or(50));
			for (id, item) in system.items.iter() {
				if search_params.matches(item) {
					matched.push((id, item));
				}
				if let Some(max_results) = *result_limit {
					let spent_time = now() - start;
					if matched.len() >= max_results || spent_time >= SEARCH_BUDGET_MS as f64 {
						stopped_early = true;
						break;
					}
				}
			}

			matched.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
			let ids = matched
				.into_iter()
				.map(|(id, _)| id)
				.cloned()
				.collect::<Vec<_>>();
			Ok((ids, stopped_early)) as Result<_, ()>
		}
	});
	use_effect_with_deps(
		{
			let results = search_results.clone();
			move |_params| {
				results.run();
			}
		},
		(search_params.clone(), result_limit.clone()),
	);

	let on_search_changed = Callback::from({
		let result_limit = result_limit.clone();
		let search_params = search_params.clone();
		move |value: SearchParams| {
			result_limit.set(Some(DEFAULT_RESULT_LIMIT));
			search_params.set(value);
		}
	});
	let on_load_all_results = Callback::from({
		let result_limit = result_limit.clone();
		move |_| {
			result_limit.set(None);
		}
	});

	let found_item_listings = match (
		search_params.is_empty(),
		search_results.loading,
		&search_results.data,
	) {
		(true, _, _) => html! {},
		(_, true, _) => html! {
			<div class="d-flex justify-content-center">
				<div class="spinner-border" role="status">
					<span class="visually-hidden">{"Loading..."}</span>
				</div>
			</div>
		},
		(_, false, None) => html! {
			<div class="text-center">
				{"No items found"}
			</div>
		},
		(_, false, Some((items, stopped_early))) => html! {<>
			{items.iter().filter_map(|id| {
				if system.items.get(id).is_none() {
					return None;
				}
				Some(html! {
					<BrowsedItemCard id={id.clone()} />
				})
			}).collect::<Vec<_>>()}
			{match (*result_limit, stopped_early) {
				(None, _) | (Some(_), false) => html! {},
				(Some(_), true) => html! {
					<button
						type="button" class="btn btn-primary"
						onclick={on_load_all_results}
					>
						{"Load All Results"}
					</button>
				}
			}}
		</>},
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Item Browser"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<SearchInput on_change={on_search_changed} />
			<div style="height: 600px">
				{found_item_listings}
			</div>
		</div>
	</>}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct SearchParams {
	text: String,
}
impl SearchParams {
	fn is_empty(&self) -> bool {
		self.text.is_empty()
	}

	fn matches(&self, item: &Item) -> bool {
		if item.name.to_lowercase().contains(&self.text.to_lowercase()) {
			return true;
		}
		false
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct SearchInputProps {
	pub on_change: Callback<SearchParams>,
}
#[function_component]
pub fn SearchInput(SearchInputProps { on_change }: &SearchInputProps) -> Html {
	let params = use_state(|| SearchParams::default());

	let set_params_text = Callback::from({
		let params = params.clone();
		let on_change = on_change.clone();
		move |text: String| {
			if params.text == text {
				return;
			}
			let mut new_params = (*params).clone();
			new_params.text = text;
			params.set(new_params.clone());
			on_change.emit(new_params);
		}
	});

	let oninput = Callback::from({
		let set_params_text = set_params_text.clone();
		move |evt: InputEvent| {
			let Some(value) = evt.input_value() else { return; };
			set_params_text.emit(value);
		}
	});

	html! {
		<div class="input-group mb-2">
			<span class="input-group-text"><i class="bi bi-search"/></span>
			<input
				type="text" class="form-control"
				placeholder="Search item names, types, rarities, or tags"
				{oninput}
			/>
		</div>
	}
}

#[function_component]
fn BrowsedItemCard(SystemItemProps { id }: &SystemItemProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();

	let add_item = Callback::from({
		let state = state.clone();
		let system = system.clone();
		move |(id, container_id): (SourceId, Option<Vec<Uuid>>)| {
			let Some(item) = system.items.get(&id).cloned() else { return; };
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				persistent.inventory.insert_to(item, &container_id);
				None
			}));
		}
	});

	let Some(item) = system.items.get(id) else { return html! {}; };
	let card_id = format!(
		"{}{}{}",
		match &id.module {
			None => String::new(),
			Some(ModuleId::Local { name }) => format!("{name}_"),
			Some(ModuleId::Github {
				user_org,
				repository,
			}) => format!("{user_org}_{repository}_"),
		},
		{
			let path = id.path.with_extension("");
			path.to_str().unwrap().replace("/", "_")
		},
		match id.node_idx {
			0 => String::new(),
			idx => format!("_{idx}"),
		}
	);
	let add_item = add_item.reform({
		let id = id.clone();
		move |container_id| (id.clone(), container_id)
	});
	html! {
		<CollapsableCard
			id={card_id}
			header_content={{
				html! {<>
					<span>{item.name.clone()}</span>
					<AddItemButton
						root_classes={"ms-auto"} btn_classes={classes!("btn-theme", "btn-xs")}
						operation={AddItemOperation::Add}
						on_click={add_item}
					/>
				</>}
			}}
		>
			{item_body(item, &state, None)}
			<AddItemActions id={id.clone()} />
		</CollapsableCard>
	}
}

#[function_component]
fn AddItemActions(SystemItemProps { id }: &SystemItemProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let auto_exchange = state.persistent().settings.currency_auto_exchange;
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let amt_to_add = use_state_eq(|| 1u32);
	let amt_to_buy = use_state_eq(|| 1u32);
	let Some(item) = system.items.get(id) else { return html! {}; };

	let add_items = Callback::from({
		let id = id.clone();
		let state = state.clone();
		let system = system.clone();
		move |(amount, cost, container_id): (u32, Wallet, Option<Vec<Uuid>>)| {
			let Some(mut item) = system.items.get(&id).cloned() else { return; };
			let items = if let ItemKind::Simple { count } = &mut item.kind {
				*count *= amount;
				vec![item]
			} else {
				let mut items = Vec::with_capacity(amount as usize);
				items.resize(amount as usize, item);
				items
			};
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				if !cost.is_empty() {
					persistent
						.inventory
						.wallet_mut()
						.remove(cost, auto_exchange);
				}
				for item in items {
					persistent.inventory.insert_to(item, &container_id);
				}
				None
			}));
		}
	});

	let section_add_amt = match &item.kind {
		ItemKind::Simple { count: batch_size } => Some({
			let amt_state = amt_to_add.clone();
			let set_amt = Callback::from({
				let amt_state = amt_state.clone();
				move |value: u32| {
					amt_state.set(value.clamp(1, 999));
				}
			});
			let onchange = Callback::from({
				let set_amt = set_amt.clone();
				move |evt: web_sys::Event| {
					let Some(value) = evt.input_value_t::<u32>() else { return; };
					set_amt.emit(value);
				}
			});
			let increment = Callback::from({
				let set_amt = set_amt.clone();
				let amt_state = amt_state.clone();
				move |_| {
					set_amt.emit(amt_state.saturating_add(1));
				}
			});
			let decrement = Callback::from({
				let set_amt = set_amt.clone();
				let amt_state = amt_state.clone();
				move |_| {
					set_amt.emit(amt_state.saturating_sub(1));
				}
			});
			let on_confirm = Callback::from({
				let add_items = add_items.clone();
				let amt_state = amt_state.clone();
				move |container_id| {
					add_items.emit((*amt_state, Wallet::default(), container_id));
					amt_state.set(1);
				}
			});
			html! {<>
				<div class="input-group item-add-amount mt-3">
					<span class="input-group-text title">{"Amount to Add"}</span>
					<button type="button" class="btn btn-outline-theme dec" onclick={decrement}><i class="bi bi-dash" /></button>
					<input
						class="form-control text-center"
						type="number"
						min="1" value={(*amt_state).to_string()}
						onkeydown={validate_uint_only()}
						onchange={onchange}
					/>
					<button type="button" class="btn btn-outline-theme inc" onclick={increment}><i class="bi bi-plus" /></button>
					<span class="input-group-text spacer" />
					<AddItemButton
						root_classes={"submit"} btn_classes={classes!("btn-theme", "btn-xs")}
						operation={AddItemOperation::Add}
						amount={*amt_state} on_click={on_confirm}
					/>
				</div>
				<div class="form-text">
					{match (*amt_state, *batch_size) {
						(1, 1) => format!("Add 1 item to your equipment."),
						(n, 1) | (1, n) => format!("Add {n} items to your equipment."),
						(n, b) => format!("Add {n} batches of {b} items to your equipment."),
					}}
				</div>
			</>}
		}),
		_ => None,
	};
	let section_purchase = match item.worth.is_empty() {
		false => Some({
			let batch_size = match &item.kind {
				ItemKind::Simple { count } => *count,
				_ => 1,
			};
			let amt_state = amt_to_buy.clone();
			let set_amt = Callback::from({
				let amt_state = amt_state.clone();
				move |value: u32| {
					amt_state.set(value.clamp(1, 999));
				}
			});
			let onchange = Callback::from({
				let set_amt = set_amt.clone();
				move |evt: web_sys::Event| {
					let Some(value) = evt.input_value_t::<u32>() else { return; };
					set_amt.emit(value);
				}
			});
			let increment = Callback::from({
				let set_amt = set_amt.clone();
				let amt_state = amt_state.clone();
				move |_| {
					set_amt.emit(amt_state.saturating_add(1));
				}
			});
			let decrement = Callback::from({
				let set_amt = set_amt.clone();
				let amt_state = amt_state.clone();
				move |_| {
					set_amt.emit(amt_state.saturating_sub(1));
				}
			});
			let purchase_cost = {
				let mut cost = item.worth * (*amt_state as u64);
				if auto_exchange {
					cost.normalize();
				}
				cost
			};
			let on_confirm = Callback::from({
				let add_items = add_items.clone();
				let amt_state = amt_state.clone();
				move |container_id| {
					add_items.emit((*amt_state, purchase_cost, container_id));
					amt_state.set(1);
				}
			});
			let not_enough_in_wallet = !state
				.inventory()
				.wallet()
				.contains(&purchase_cost, auto_exchange);
			html! {<>
				<div class="input-group item-add-amount mt-3">
					<span class="input-group-text title">{"Purchase"}</span>
					<button type="button" class="btn btn-outline-theme dec" onclick={decrement}><i class="bi bi-dash" /></button>
					<input
						class="form-control text-center"
						type="number"
						min="1" value={(*amt_state).to_string()}
						onkeydown={validate_uint_only()}
						onchange={onchange}
					/>
					<button type="button" class="btn btn-outline-theme inc" onclick={increment}><i class="bi bi-plus" /></button>
					<span class="input-group-text spacer" />
					<AddItemButton
						root_classes={"submit"} btn_classes={classes!("btn-theme", "btn-xs")}
						amount={*amt_state} on_click={on_confirm}
						operation={AddItemOperation::Buy}
						disabled={not_enough_in_wallet}
					/>
				</div>
				<div class="form-text">
					{match (*amt_state, batch_size) {
						(1, 1) => format!("Buy 1 item for "),
						(n, 1) | (1, n) => format!("Buy {n} items for "),
						(n, b) => format!("Buy {n} batches of {b} items for "),
					}}
					<span><WalletInline wallet={purchase_cost} /></span>
				</div>
			</>}
		}),
		true => None,
	};
	html! {<>
		{section_add_amt.unwrap_or_default()}
		{section_purchase.unwrap_or_default()}
	</>}
}
