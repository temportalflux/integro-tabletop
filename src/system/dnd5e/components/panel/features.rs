use crate::{
	components::{Tag, Tags},
	data::ContextMut,
	path_map::PathMap,
	system::dnd5e::data::{character::Character, BoxedFeature},
	utility::{Evaluator, MutatorGroup},
};
use std::path::{Path, PathBuf};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn Features() -> Html {
	let state = use_context::<ContextMut<Character>>().unwrap();
	let sort_order_alpha = use_state(|| true);

	let features = match *sort_order_alpha {
		true => {
			let features = {
				let mut features = state.features().as_vec();
				features.sort_by(|(_, a), (_, b)| a.inner().name.cmp(&b.inner().name));
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
		false => make_section_contents(PathBuf::new(), state.features()),
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

fn make_section(parent: &Path, title: &String, container: &PathMap<BoxedFeature>) -> Html {
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

fn make_section_contents(parent: PathBuf, container: &PathMap<BoxedFeature>) -> Html {
	let top_level_features = container
		.iter_values()
		.map(|feat| {
			html! {
				<FeatureBlock feature={feat.clone()} parent={parent.to_path_buf()} show_parent={false} />
			}
		})
		.collect::<Vec<_>>();
	let sections = container
		.iter_children()
		.map(|(key, children)| make_section(&parent, key, children))
		.collect::<Vec<_>>();
	html! {<>
		{top_level_features}
		{sections}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct FeatureBlockProps {
	parent: PathBuf,
	feature: BoxedFeature,
	show_parent: bool,
}
#[function_component]
fn FeatureBlock(
	FeatureBlockProps {
		parent,
		feature,
		show_parent,
	}: &FeatureBlockProps,
) -> Html {
	use convert_case::{Case, Casing};
	let state = use_context::<ContextMut<Character>>().unwrap();
	let feat_data_path = match feature.inner().id() {
		Some(id) => parent.join(&id),
		None => parent.clone(),
	};
	let selected_value_map = state.selected_values_in(&feat_data_path);
	let missing_selections = state.missing_selections_in(&feat_data_path);

	let name = feature.inner().name.to_case(Case::Title);
	let mut description = feature.inner().description.clone();
	let mut selected_values = Vec::new();
	if let Some(value_map) = selected_value_map {
		let values = value_map.as_vec();
		selected_values = values.iter().map(|(_, value)| (*value).clone()).collect();
		description =
			values
				.into_iter()
				.fold(feature.inner().description.clone(), |desc, (key, value)| {
					let key = key.to_str().unwrap();
					let search_key = format!("{{{key}}}");
					desc.replace(&search_key, value)
				});
	}
	description = missing_selections.iter().fold(description, |desc, key| {
		let key = key.to_str().unwrap();
		let search_key = format!("{{{key}}}");
		desc.replace(
			&search_key,
			match feature.inner().get_missing_selection_text_for(key) {
				Some(text) => text.as_str(),
				None => "MISSING_SELECTION",
			},
		)
	});

	let consumed_uses = use_state(|| 0);

	let uses = match &feature.inner().limited_uses {
		Some(limited_uses) => match limited_uses.max_uses.evaluate(&*state) {
			Some(max_uses) => {
				let toggle_use = Callback::from({
					let consumed_uses = consumed_uses.clone();
					move |evt: web_sys::Event| {
						let Some(target) = evt.target() else { return; };
						let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
						let consume_use = input.checked();
						if consume_use {
							consumed_uses.set(*consumed_uses + 1);
						} else {
							consumed_uses.set(*consumed_uses - 1);
						}
					}
				});
				let use_checkboxes = (0..max_uses)
					.map(|idx| {
						html! {
							<input
								class={"form-check-input"} type={"checkbox"}
								checked={idx < *consumed_uses}
								onclick={Callback::from(|evt: web_sys::MouseEvent| evt.stop_propagation())}
								onchange={toggle_use.clone()}
							/>
						}
					})
					.collect::<Vec<_>>();
				html! {
					<span>
						{use_checkboxes}
						{match &limited_uses.reset_on {
							Some(rest) => html! { <span>{"/"}{format!("{:?} Rest", rest)}</span> },
							None => html! {},
						}}
					</span>
				}
			}
			None => html! {},
		},
		None => html! {},
	};

	html! {
		<div style="border-width: 0; border-bottom: 1px; border-style: solid; border-color: var(--theme-frame-color-muted);">
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
			<div style="white-space: pre-line;">
				{description}
			</div>
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
			{uses}
		</div>
	}
}
