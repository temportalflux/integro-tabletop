use crate::{
	data::ContextMut,
	path_map::PathMap,
	system::dnd5e::{character::State, BoxedFeature},
};
use yew::prelude::*;

#[function_component]
pub fn Features() -> Html {
	let state = use_context::<ContextMut<State>>().unwrap();

	/* TODO: Alphabetical vs Hierarchial presentation
	let feature_names = state.features().as_vec().into_iter().map(|(path, feat)| {
		html! {
			<div>
				{path.display().to_string()}{": "}{feat.inner().name.clone()}
			</div>
		}
	}).collect::<Vec<_>>();
	*/

	html! {<>
		{make_section_contents(state.features())}
	</>}
}

fn make_section((title, container): (&String, &PathMap<BoxedFeature>)) -> Html {
	use convert_case::{Case, Casing};
	html! {
		<div>
			{title.to_case(Case::Title)}
			<div class="d-flex" style="padding-left: 5px;">
				<div class="vr" />
				<div style="padding-left: 5px;">
					{make_section_contents(container)}
				</div>
			</div>
		</div>
	}
}

fn make_section_contents(container: &PathMap<BoxedFeature>) -> Html {
	let top_level_features = container
		.iter_values()
		.map(make_feature_block)
		.collect::<Vec<_>>();
	let sections = container
		.iter_children()
		.map(make_section)
		.collect::<Vec<_>>();
	html! {<>
		{top_level_features}
		{sections}
	</>}
}

fn make_feature_block(feature: &BoxedFeature) -> Html {
	use convert_case::{Case, Casing};
	let name = feature.inner().name.to_case(Case::Title);
	html! {
		<div>{name}</div>
	}
}
