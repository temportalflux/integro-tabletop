use yew::prelude::*;

#[function_component]
pub fn Proficiencies() -> Html {
	html! {
		<div id="proficiencies-container" class="card" style="max-width: 200px; margin: 0 auto; border-color: var(--theme-frame-color);">
			<div class="card-body" style="padding: 5px;">
				<h5 class="card-title text-center" style="font-size: 0.8rem;">{"Proficiencies"}</h5>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Languages"}</h6>
					<span>{"Common, Gnomish, Sylvan, Undercommon"}</span>
				</div>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Armor"}</h6>
					<span>{"None"}</span>
				</div>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Weapons"}</h6>
					<span>{"Crossbow, Light, Dagger, Dart, Quarterstaff, Sling"}</span>
				</div>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Tools"}</h6>
					<span>{"Cartographer's Tools"}</span>
				</div>
			</div>
		</div>
	}
}
