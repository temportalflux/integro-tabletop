use crate::{data::ContextMut, system::dnd5e::character::State, bootstrap::components::Tooltip};
use yew::prelude::*;

#[function_component]
pub fn Proficiencies() -> Html {
	let state = use_context::<ContextMut<State>>().unwrap();

	let languages = {
		let languages = state.languages();
		let lang_count = languages.len();
		languages
			.iter()
			.enumerate()
			.map(|(idx, (lang, srcs))| (lang, srcs, idx == lang_count - 1))
			.fold(Vec::new(), |mut html, (lang, sources, is_last)| {
				let tooltip = (!sources.is_empty()).then(|| {
					sources.iter().filter_map(|path| crate::data::as_feature_path_text(&path)).map(|path| {
						format!("<div>{}</div>", path)
					}).collect::<Vec<_>>().join("\n")
				});
				html.push(html! {
					<span>
						<Tooltip tag="span" content={tooltip} use_html={true}>
							{lang.clone()}
						</Tooltip>
						{match is_last {
							false => ", ",
							true => "",
						}}
					</span>
				});
				html
			})
	};

	html! {
		<div id="proficiencies-container" class="card" style="max-width: 200px; margin: 0 auto; border-color: var(--theme-frame-color);">
			<div class="card-body" style="padding: 5px;">
				<h5 class="card-title text-center" style="font-size: 0.8rem;">{"Proficiencies"}</h5>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Languages"}</h6>
					<span>{languages}</span>
				</div>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Armor"}</h6>
					<span>{"TODO: None"}</span>
				</div>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Weapons"}</h6>
					<span>{"TODO: Crossbow, Light, Dagger, Dart, Quarterstaff, Sling"}</span>
				</div>
				<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
					<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Tools"}</h6>
					<span>{"TODO: Cartographer's Tools"}</span>
				</div>
			</div>
		</div>
	}
}
