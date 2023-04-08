use crate::system::{
	core::ModuleId,
	dnd5e::{components::{editor::CollapsableCard, panel::item_body}, data::item::Item, DnD5e},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn BrowseModal() -> Html {
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let relevant_item_ids = use_state(|| Vec::new());

	let on_search_changed = Callback::from({
		let system = system.clone();
		let relevant_item_ids = relevant_item_ids.clone();
		move |value: SearchParams| {
			log::debug!(target: "inv", "Searching for: {:?}", value);
			if value.is_empty() {
				relevant_item_ids.set(Vec::new());
				return;
			}
			let ids = system
				.items
				.iter()
				.filter_map(|(id, item)| value.matches(item).then_some(id))
				.cloned()
				.collect::<Vec<_>>();
			relevant_item_ids.set(ids);
		}
	});

	let items = {
		let mut items = relevant_item_ids.iter().filter_map(|id| system.items.get(id).map(|item| (id, item))).collect::<Vec<_>>();
		items.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
		items
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Item Browser"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<SearchInput on_change={on_search_changed} />
			<div style="height: 600px">
				{items.into_iter().map(|(id, item)| {
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
					html! {
						<CollapsableCard
							id={card_id}
							header_content={{
								html! {<>
									<span>{item.name.clone()}</span>
									<button
										type="button" class="btn btn-primary btn-xs ms-auto"
									>
										{"Add"}
									</button>
								</>}
							}}
						>
							{item_body(item)}
						</CollapsableCard>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>
	</>}
}

#[derive(Clone, Default, Debug)]
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
