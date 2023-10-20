use enumset::{EnumSet, EnumSetType};
use gloo_storage::Storage;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use yew::prelude::*;

#[derive(Debug, EnumSetType, Serialize, Deserialize)]
pub enum Theme {
	Auto,
	#[serde(rename = "dark")]
	Dark,
	#[serde(rename = "light")]
	Light,
}

impl Default for Theme {
	fn default() -> Self {
		Self::Auto
	}
}

impl Theme {
	/// Returns the strict enum value as a string.
	pub fn as_str(&self) -> Option<&'static str> {
		match self {
			Self::Auto => None,
			Self::Dark => Some("dark"),
			Self::Light => Some("light"),
		}
	}

	/// Returns the value that should be applied to the document for this theme.
	/// This will redirect the Auto theme to dark mode if thats the system's preference.
	/// Delegates to [`as_str`] for value representation.
	pub fn as_attribute_value(&self) -> Option<&'static str> {
		match self {
			Self::Auto => {
				match gloo_utils::window().match_media("(prefers-color-scheme: dark)") {
					// if the system prefers dark-mode, then emulate the dark theme when applying to document.
					Ok(Some(query)) if query.matches() => Theme::Dark.as_str(),
					_ => None,
				}
			}
			theme => theme.as_str(),
		}
	}

	pub fn as_icon_name(&self) -> &'static str {
		match self {
			Self::Auto => "bi-circle-half",
			Self::Dark => "bi-moon-fill",
			Self::Light => "bi-sun-fill",
		}
	}

	pub fn as_display_name(&self) -> &'static str {
		match self {
			Self::Auto => "Auto",
			Self::Dark => "Dark",
			Self::Light => "Light",
		}
	}
}

impl FromStr for Theme {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"dark" => Ok(Self::Dark),
			"light" => Ok(Self::Light),
			_ => Ok(Self::Auto),
		}
	}
}

#[function_component]
pub fn Dropdown() -> Html {
	let theme = use_state(|| gloo_storage::LocalStorage::get::<Theme>("theme").unwrap_or_default());

	// Update the theme in storage and html when the theme value has changed
	use_effect_with((*theme).clone(), move |theme| {
		log::debug!("Setting theme to {:?}", theme);
		// Write the theme to local-storage, deleting if the desired value is automatic.
		match theme {
			Theme::Auto => {
				gloo_storage::LocalStorage::delete("theme");
			}
			theme => {
				let _ = gloo_storage::LocalStorage::set("theme", theme);
			}
		}
		// Apply the theme to the document
		match theme.as_attribute_value() {
			None => {
				let _ = gloo_utils::document()
					.document_element()
					.unwrap()
					.remove_attribute("data-bs-theme");
			}
			Some(theme) => {
				let _ = gloo_utils::document()
					.document_element()
					.unwrap()
					.set_attribute("data-bs-theme", theme);
			}
		}
	});

	let onclick = {
		let theme = theme.clone();
		Callback::from(move |e: MouseEvent| {
			let Some(element) = e.target_dyn_into::<web_sys::HtmlElement>() else {
				return;
			};
			let value = element
				.get_attribute("value")
				.map(|s| Theme::from_str(&s).ok())
				.flatten()
				.unwrap_or_default();
			theme.set(value);
		})
	};

	html! {
		<li class="nav-item dropdown">
			<a class="nav-link dropdown-toggle" role="button" data-bs-toggle="dropdown" aria-expanded="false">
				<i class={format!("bi {}", theme.as_icon_name())} />
			</a>
			<div class="dropdown-menu dropdown-menu-end" style="--bs-dropdown-min-width: 0rem;">
				{EnumSet::<Theme>::all().into_iter().map(|value| html! {
					<a class="dropdown-item" value={value.as_str()} onclick={onclick.clone()}>
						<i class={format!("bi {}", value.as_icon_name())} style="margin-right: 5px;"></i>
						{value.as_display_name()}
					</a>
				}).collect::<Vec<_>>()}
			</div>
		</li>
	}
}
