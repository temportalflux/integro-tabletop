use crate::{bootstrap::components::Tooltip, system::dnd5e::components::SharedCharacter};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
struct SingleValueProps {
	title: AttrValue,
	amount: i32,
}

#[function_component]
fn SingleValue(SingleValueProps { title, amount }: &SingleValueProps) -> Html {
	html! {<div>
		<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{title.clone()}</h6>
		<div class="text-center" style="width: 100%;">
			<span style="position: relative; font-size: 26px; font-weight: 500;">
				<span>{*amount}</span>
				<span style="position: absolute; bottom: 2px; font-size: 16px; margin-left: 3px;">{"ft."}</span>
			</span>
		</div>
	</div>}
}

#[function_component]
pub fn SpeedAndSenses() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	let divider = (state.speeds().len() > 0 && state.senses().len() > 0)
		.then(|| {
			html! {
				<div class="col-auto p-0"><div class="vr" style="min-height: 100%;" /></div>
			}
		})
		.unwrap_or_else(|| html! {});
	let speed = match state.speeds().len() {
		0 => html! {},
		1 => {
			let (title, attributed) = state.speeds().iter().next().unwrap();
			let tooltip = crate::data::as_feature_paths_html_custom(
				attributed.sources().iter(),
				|(path, value)| (*value, path.as_path()),
				|value, path_str| format!("<div>{}ft. ({})</div>", value, path_str),
			);
			html! {<div class="col">
				<Tooltip content={tooltip} use_html={true}>
					<SingleValue title={format!("{title} Speed")} amount={attributed.value()} />
				</Tooltip>
			</div>}
		}
		// TODO: Walking speed should always be the first entry
		_ => html! {<div class="col">
			<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Speeds"}</h6>
			<div style="margin-left: 5px; margin-right: 5px;">
				{state.speeds().iter().map(|(title, attributed)| {
					let tooltip = crate::data::as_feature_paths_html_custom(
						attributed.sources().iter(),
						|(path, value)| (*value, path.as_path()),
						|value, path_str| {
							format!("<div>{}ft. ({})</div>", value, path_str)
						},
					);
					html! {
						<Tooltip content={tooltip} use_html={true}>
							<span class="d-flex" style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
								<span class="flex-grow-1">{title}</span>
								<span class="ps-2">{attributed.value()}{"ft."}</span>
							</span>
						</Tooltip>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>},
	};
	let senses_html = match state.senses().len() {
		0 => html! {},
		1 => {
			let (title, attributed) = state.senses().iter().next().unwrap();
			let tooltip = crate::data::as_feature_paths_html_custom(
				attributed.sources().iter(),
				|(path, value)| (*value, path.as_path()),
				|value, path_str| format!("<div>{}ft. ({})</div>", value, path_str),
			);
			html! {<div class="col">
				<Tooltip content={tooltip} use_html={true}>
					<SingleValue title={title.clone()} amount={attributed.value()} />
				</Tooltip>
			</div>}
		}
		_ => html! {<div class="col">
			<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Senses"}</h6>
			<div style="margin-left: 5px; margin-right: 5px;">
				{state.senses().iter().map(|(title, attributed)| {
					let tooltip = crate::data::as_feature_paths_html_custom(
						attributed.sources().iter(),
						|(path, value)| (*value, path.as_path()),
						|value, path_str| {
							format!("<div>{}ft. ({})</div>", value, path_str)
						},
					);
					html! {
						<Tooltip content={tooltip} use_html={true}>
							<span class="d-flex" style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
								<span class="flex-grow-1">{title}</span>
								<span class="ps-2">{attributed.value()}{"ft."}</span>
							</span>
						</Tooltip>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>},
	};

	let width = match (state.speeds().len(), state.senses().len()) {
		(0, 1) | (1, 0) => "120px",
		(n, 0) | (0, n) if n > 1 => "160px",
		(1, 1) => "240px",
		_ => "100%",
	};

	html! {
		<div class="card my-1" style={format!("width: {width};")}>
			<div class="card-body" style="padding: 5px 5px;">
				<div class="row" style="--bs-gutter-x: 0;">
					{speed}
					{divider}
					{senses_html}
				</div>
			</div>
		</div>
	}
}
