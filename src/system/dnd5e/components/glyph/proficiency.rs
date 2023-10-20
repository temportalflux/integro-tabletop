use crate::system::dnd5e::{components::GeneralProp, data::proficiency};
use yew::prelude::*;

#[function_component]
pub fn ProficiencyLevel(GeneralProp { value }: &GeneralProp<proficiency::Level>) -> Html {
	match value {
		proficiency::Level::None => html! { <i class="bi bi-circle" /> },
		proficiency::Level::HalfDown | proficiency::Level::HalfUp => {
			html! { <i class="bi bi-circle-half" style="color: var(--theme-frame-color);" /> }
		}
		proficiency::Level::Full => {
			html! { <i class="bi bi-circle-fill" style="color: var(--theme-frame-color);" /> }
		}
		proficiency::Level::Double => {
			html! { <i class="bi bi-record-circle" style="color: var(--theme-frame-color);" /> }
		}
	}
}
