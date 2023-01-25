use gloo_storage::Storage;
use wasm_bindgen::JsCast;
use web_sys::{HtmlSelectElement};
use yew::prelude::*;

#[function_component]
fn ThemeToggle() -> Html {
	let theme = use_state(|| gloo_storage::LocalStorage::get::<String>("theme").unwrap_or_default());

	// Update the theme in storage and html when the element is reloaded
	if theme.is_empty() {
		gloo_storage::LocalStorage::delete("theme");
		let _ = gloo_utils::document().document_element().unwrap().remove_attribute("data-theme");
	}
	else {
		let _ = gloo_utils::document().document_element().unwrap().set_attribute("data-theme", &*theme);
		let _ = gloo_storage::LocalStorage::set("theme", (*theme).clone());
	}

	// When select element changes, set the value in the state so this element reloads.
	let onchange = {
		let theme = theme.clone();
		Callback::from(move |e: Event| {
			if let Some(select) = e.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
				theme.set(select.value());
			}
		})
	};
	
	html! {
		<select class="select w-full max-w-xs" {onchange}>
			<option value="" selected={*theme == ""}>{"Default"}</option>
			<option value="dark" selected={*theme == "dark"}>{"Dark"}</option>
			<option value="light" selected={*theme == "light"}>{"Light"}</option>
		</select>
	}
}

#[function_component]
fn App() -> Html {
	html! {<>
		<h1 class="text-3xl font-bold underline">
			{"Hello World!"}
		</h1>
		<ThemeToggle />
	</>}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<App>::new().render();
}
