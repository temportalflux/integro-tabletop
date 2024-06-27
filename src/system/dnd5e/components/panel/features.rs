use crate::{
	components::{Tag, Tags},
	page::characters::sheet::{joined::editor::description, CharacterHandle},
	path_map::PathMap,
	system::dnd5e::data::Feature,
};
use std::path::{Path, PathBuf};
use yew::prelude::*;

#[function_component]
pub fn Features() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let sort_order_alpha = use_state(|| true);

	let features = match *sort_order_alpha {
		true => {
			let features = {
				let mut features = state.features().path_map.as_vec();
				features.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
				features
			};
			let features = features
				.into_iter()
				.map(|(path, feat)| {
					html! {
						<FeatureBlock feature={feat.clone()} parent={path} show_parent={true} />
					}
				})
				.collect::<Vec<_>>();
			html! {<>{features}</>}
		}
		false => make_section_contents(PathBuf::new(), &state.features().path_map),
	};

	html! {<>
		<Tags>
			<span style="margin-right: 10px;">{"[TMP] Order:"}</span>
			<Tag active={*sort_order_alpha == true} on_click={Callback::from({
				let sort_order_alpha = sort_order_alpha.clone();
				move |_| {
					sort_order_alpha.set(true);
				}
			})}>{"Alpha"}</Tag>
			<Tag active={*sort_order_alpha == false} on_click={Callback::from({
				let sort_order_alpha = sort_order_alpha.clone();
				move |_| {
					sort_order_alpha.set(false);
				}
			})}>{"Path"}</Tag>
		</Tags>
		<div style="height: 480px; overflow-y: scroll;">
			<div style="margin: 5px;">
				{features}
			</div>
		</div>
	</>}
}

fn make_section(parent: &Path, title: &String, container: &PathMap<Feature>) -> Html {
	use convert_case::{Case, Casing};
	html! {
		<div>
			<h4>{title.to_case(Case::Title)}</h4>
			<div class="d-flex" style="padding-left: 5px;">
				<div class="vr" />
				<div style="margin-left: 5px; padding: 5px;">
					{make_section_contents(parent.join(title), container)}
				</div>
			</div>
		</div>
	}
}

fn make_section_contents(parent: PathBuf, container: &PathMap<Feature>) -> Html {
	let top_level_features = container
		.iter_values()
		.map(|feat| {
			html! {
				<FeatureBlock feature={feat.clone()} parent={parent.to_path_buf()} show_parent={false} />
			}
		})
		.collect::<Vec<_>>();
	let sections =
		container.iter_children().map(|(key, children)| make_section(&parent, key, children)).collect::<Vec<_>>();
	html! {<>
		{top_level_features}
		{sections}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct FeatureBlockProps {
	parent: PathBuf,
	feature: Feature,
	show_parent: bool,
}
#[function_component]
fn FeatureBlock(FeatureBlockProps { parent, feature, show_parent }: &FeatureBlockProps) -> Html {
	use convert_case::{Case, Casing};
	let state = use_context::<CharacterHandle>().unwrap();
	let feat_data_path = feature.get_display_path();
	let selected_value_map = state.selected_values_in(&feat_data_path);

	let name = feature.name.to_case(Case::Title);
	let mut selected_values = Vec::new();
	if let Some(value_map) = selected_value_map {
		selected_values = value_map.as_vec().iter().map(|(_, value)| (*value).clone()).collect();
	}

	html! {
		<div class="border-bottom border-bottom-theme-muted">
			<span>
				<h5 class="d-inline" style="margin-right: 5px;">{name}</h5>
				{match (*show_parent, crate::data::as_feature_path_text(parent)) {
					(true, Some(text)) => {
						html! {
							<span style="font-size: 14px;">{"("}{text}{")"}</span>
						}
					}
					_ => html! {},
				}}
			</span>
			{description(&feature.description, true, false)}
			{match selected_values.len() {
				0 => html! {},
				_ => {
					let list_string = selected_values.iter().fold(String::new(), |list, value| {
						let separator = (list.len() > 0).then(|| ", ").unwrap_or_default();
						format!("{list}{separator}{value}")
					});
					html! {
						<div>
							{"Selections: "}{list_string}
						</div>
					}
				}
			}}
		</div>
	}
}
