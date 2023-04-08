use crate::system::{
	core::{ModuleId, SourceId},
	dnd5e::{
		components::{editor::CollapsableCard, panel::item_body, SharedCharacter},
		data::{item::Item, character::Persistent},
		DnD5e,
	},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
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
	let state = use_context::<SharedCharacter>().unwrap();
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
			let ids = matched.into_iter().map(|(id, _)| id).cloned().collect::<Vec<_>>();
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

	let add_item = Callback::from({
		let state = state.clone();
		let system = system.clone();
		move |id: SourceId| {
			let Some(item) = system.items.get(&id).cloned() else { return; };
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				persistent.inventory.insert(item);
				None
			}));
		}
	});

	let found_item_listings = match (search_params.is_empty(), search_results.loading, &search_results.data) {
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
				let Some(item) = system.items.get(&id) else { return None; };
				let card_id = format!(
					"{}{}{}",
					match &id.module {
						None => String::new(),
						Some(ModuleId::Local { name }) => format!("{name}_"),
						Some(ModuleId::Github { user_org, repository }) => format!("{user_org}_{repository}_"),
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
					move |_| id.clone()
				});
				Some(html! {
					<CollapsableCard
						id={card_id}
						header_content={{
							html! {<>
								<span>{item.name.clone()}</span>
								<button
									type="button" class="btn btn-primary btn-xs ms-auto"
									onclick={add_item}
								>
									{"Add"}
								</button>
							</>}
						}}
					>
						{item_body(item)}
					</CollapsableCard>
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
			let Some(target) = evt.target() else { return; };
			let Some(element) = target.dyn_ref::<HtmlInputElement>() else { return; };
			set_params_text.emit(element.value());
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
