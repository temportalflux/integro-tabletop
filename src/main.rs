use std::str::FromStr;

use enumset::{EnumSet, EnumSetType};
use gloo_storage::Storage;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(EnumSetType, Serialize, Deserialize)]
enum Theme {
	#[serde(rename = "dark")]
	Dark,
	#[serde(rename = "light")]
	Light,
}
impl std::fmt::Debug for Theme {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dark => write!(f, "dark"),
			Self::Light => write!(f, "light"),
		}
	}
}
impl std::fmt::Display for Theme {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dark => write!(f, "Dark"),
			Self::Light => write!(f, "Light"),
		}
	}
}
impl FromStr for Theme {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"dark" => Ok(Self::Dark),
			"light" => Ok(Self::Light),
			_ => Err(()),
		}
	}
}

#[function_component]
fn ThemeToggle() -> Html {
	let theme = use_state(|| gloo_storage::LocalStorage::get::<Theme>("theme").ok());

	// Update the theme in storage and html when the theme value has changed
	use_effect_with_deps(
		move |theme| {
			log::debug!("Setting theme to {:?}", theme);
			match theme.as_ref() {
				Some(theme) => {
					let _ = gloo_utils::document()
						.document_element()
						.unwrap()
						.set_attribute("data-theme", &format!("{theme:?}"));
					let _ = gloo_storage::LocalStorage::set("theme", (*theme).clone());
				}
				None => {
					gloo_storage::LocalStorage::delete("theme");
					let _ = gloo_utils::document()
						.document_element()
						.unwrap()
						.remove_attribute("data-theme");
				}
			}
		},
		(*theme).clone(),
	);

	// When select element changes, set the value in the state so this element reloads.
	let onchange = {
		let theme = theme.clone();
		Callback::from(move |e: Event| {
			if let Some(select) = e
				.target()
				.and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
			{
				theme.set(Theme::from_str(&select.value()).ok());
			}
		})
	};

	html! {
		<select class="select w-full max-w-xs" {onchange}>
			<option selected={*theme == None}>{"Default"}</option>
			{EnumSet::<Theme>::all().into_iter().map(|value| html! {
				<option value={format!("{value:?}")} selected={*theme == Some(value)}>{format!("{value}")}</option>
			}).collect::<Vec<_>>()}
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
