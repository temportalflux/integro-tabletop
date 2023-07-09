use yew::prelude::*;
use crate::system::dnd5e::components::panel::{AgePronouns, SizeHeightWeight, Personality, AppearanceEditor};

#[function_component]
pub fn Page() -> Html {
	html! {
		<div class="m-2">
			<AgePronouns />
			<div class="hr my-2" />
			<SizeHeightWeight />
			<div class="hr my-2" />
			<Personality />
			<div class="hr my-2" />
			<AppearanceEditor />
		</div>
	}
}
