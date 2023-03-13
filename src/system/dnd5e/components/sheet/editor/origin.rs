use crate::system::dnd5e::{components::SharedCharacter, data::{character::Persistent, Lineage}, DnD5e};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

static HELP_TEXT: &'static str = "Lineages and Upbingings are a replacement for races. \
They offer an expanded set of options around traits and features granted from \
the parents and community your character comes from.";

#[function_component]
pub fn OriginTab() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let use_lineages = use_state_eq(|| true);

	let toggle_lineages = Callback::from({
		let use_lineages = use_lineages.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			use_lineages.set(input.checked());
		}
	});

	html! {<>
		<div class="form-check form-switch m-2">
			<label for="useLineages" class="form-check-label">{"Use Lineages & Upbringings"}</label>
			<input  id="useLineages" class="form-check-input"
				type="checkbox" role="switch"
				checked={*use_lineages}
				onchange={toggle_lineages}
			/>
			<div id="useLineagesHelpBlock" class="form-text">{HELP_TEXT}</div>
		</div>
		<h4>{"Lineages"}</h4>
		<div class="accordion m-2" id="all-lineages">
			{system.lineages.iter().map(|(_source_id, lineage)| html! {
				<LineageItem lineage={lineage.clone()} />
			}).collect::<Vec<_>>()}
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct LineageItemProps {
	lineage: Lineage,
}

#[function_component]
fn LineageItem(LineageItemProps { lineage }: &LineageItemProps) -> Html {
	use convert_case::{Case, Casing};
	let id = lineage.name.to_case(Case::Kebab);
	html! {
		<div class="accordion-item">
			<h2 class="accordion-header">
				<button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target={format!("#{id}")}>
					{lineage.name.clone()}
				</button>
			</h2>
			<div {id} class="accordion-collapse collapse" data-bs-parent="#all-lineages">
				<div class="accordion-body" style="white-space: pre-line;">
					{lineage.description.clone()}
				</div>
			</div>
		</div>
	}
}
