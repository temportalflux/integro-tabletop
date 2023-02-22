use crate::{
	bootstrap::components::Tooltip,
	data::ContextMut,
	system::dnd5e::character::{AttributedValueMap, Character},
};
use yew::prelude::*;

#[function_component]
pub fn Proficiencies() -> Html {
	let state = use_context::<ContextMut<Character>>().unwrap();
	let proficiencies = state.other_proficiencies();
	html! {
		<div id="proficiencies-container" class="card" style="max-width: 200px; margin: 0 auto; border-color: var(--theme-frame-color);">
			<div class="card-body" style="padding: 5px;">
				<h5 class="card-title text-center" style="font-size: 0.8rem;">{"Proficiencies"}</h5>
				{make_proficiencies_section("Languages", &proficiencies.languages)}
				{make_proficiencies_section("Armor", &proficiencies.armor)}
				{make_proficiencies_section("Weapons", &proficiencies.weapons)}
				{make_proficiencies_section("Tools", &proficiencies.tools)}
			</div>
		</div>
	}
}

fn make_proficiencies_section<T>(title: &str, values: &AttributedValueMap<T>) -> Html
where
	T: ToString,
{
	let count = values.len();
	let iter = values.iter().enumerate();
	let iter = iter.map(|(idx, (value, srcs))| (value, srcs, idx == count - 1));
	let items = iter.fold(
		Vec::with_capacity(count),
		|mut html, (value, sources, is_last)| {
			let tooltip = crate::data::as_feature_paths_html(sources.iter());
			html.push(html! {
				<span>
					<Tooltip tag="span" content={tooltip} use_html={true}>
						{value.to_string()}
					</Tooltip>
					{match is_last {
						false => ", ",
						true => "",
					}}
				</span>
			});
			html
		},
	);
	html! {
		<div class="proficiency-section">
			<h6>{title}</h6>
			<span>{match items.is_empty() {
				true => html! { {"None"} },
				false => html! {<> {items} </>},
			}}</span>
		</div>
	}
}
