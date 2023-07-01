use crate::{
	components::modal,
	page::characters::sheet::CharacterHandle,
	system::dnd5e::data::bounded::{BoundKind, BoundedValue},
};
use enumset::EnumSet;
use std::{collections::BTreeMap, path::PathBuf};
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
	let state = use_context::<CharacterHandle>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

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
			let (title, bounded) = state.speeds().iter().next().unwrap();
			html! {<div class="col">
				<SingleValue title={format!("{title} Speed")} amount={bounded.value()} />
			</div>}
		}
		// TODO: Walking speed should always be the first entry
		_ => html! {<div class="col">
			<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Speeds"}</h6>
			<div style="margin-left: 5px; margin-right: 5px;">
				{state.speeds().iter().map(|(title, bounded)| {
					html! {
						<span class="d-flex" style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
							<span class="flex-grow-1">{title}</span>
							<span class="ps-2">{bounded.value()}{"ft."}</span>
						</span>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>},
	};
	let senses_html = match state.senses().len() {
		0 => html! {},
		1 => {
			let (title, bounded) = state.senses().iter().next().unwrap();
			html! {<div class="col">
				<SingleValue title={title.clone()} amount={bounded.value()} />
			</div>}
		}
		_ => html! {<div class="col">
			<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Senses"}</h6>
			<div style="margin-left: 5px; margin-right: 5px;">
				{state.senses().iter().map(|(title, bounded)| {
					html! {
						<span class="d-flex" style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
							<span class="flex-grow-1">{title}</span>
							<span class="ps-2">{bounded.value()}{"ft."}</span>
						</span>
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

	let onclick = modal_dispatcher.callback({
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<Modal />},
				..Default::default()
			})
		}
	});

	html! {
		<div class="card my-1" style={format!("width: {width};")} {onclick}>
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

static SENSE_DESC: [(&'static str, &'static str); 3] = [
	(
		"Blindsight",
		"A creature with blindsight can perceive its surroundings \
		without relying on sight, within a specific radius.
		Creatures without eyes, such as grimlocks and gray oozes, typically have this special sense, \
		as do creatures with echolocation or heightened senses, such as bats and true dragons.
		If a creature is naturally blind, it has a parenthetical note to this effect, indicating that \
		the radius of its blindsight defines the maximum range of its perception.",
	),
	(
		"Darkvision",
		"A creature with darkvision can see in the dark within a specific radius. \
		The creature can see in dim light within the radius as if it were bright light, \
		and in darkness as if it were dim light. The creature can't discern color in darkness, \
		only shades of gray. Many creatures that live underground have this special sense.",
	),
	(
		"Tremorsense",
		"A creature with tremorsense can detect and pinpoint the origin of vibrations \
		within a specific radius, provided that the creature and the source of the \
		vibrations are in contact with the same ground or substance.
		Tremorsense can't be used to detect flying or incorporeal creatures. \
		Many burrowing creatures, such as ankhegs, have this special sense.",
	),
];

#[function_component]
fn Modal() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Speeds & Senses"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{state.speeds().iter().map(|(name, bounded)| {
				bounded_value("Speed", &name, bounded)
			}).collect::<Vec<_>>()}
			{state.senses().iter().map(|(name, bounded)| {
				bounded_value("Sense", &name, bounded)
			}).collect::<Vec<_>>()}

			<div>
				<h6>{"Additional Information"}</h6>
				{SENSE_DESC.iter().map(|(title, desc)| html! {
					<div>
						<strong>{*title}{". "}</strong>
						<span class="text-block" style="font-size: 14px;">
							{*desc}
						</span>
					</div>
				}).collect::<Vec<_>>()}
			</div>

		</div>
	</>}
}

fn bounded_value(kind: &str, name: &str, bounded: &BoundedValue) -> Html {
	let bound_kinds = {
		let mut kinds = EnumSet::<BoundKind>::all().into_iter().collect::<Vec<_>>();
		kinds.sort();
		kinds
	};
	let bound_sources = bound_kinds
		.into_iter()
		.filter_map(|kind| {
			let sources = bounded.argument(kind);
			(!sources.is_empty()).then_some((kind, sources))
		})
		.collect::<Vec<_>>();
	html! {
		html! {<div class="mb-2">
			<h4>{name}{" ("}{kind}{")"}</h4>
			{bound_sources.into_iter().map(|(kind, sources)| bound_section(kind, sources)).collect::<Vec<_>>()}
		</div>}
	}
}

fn bound_section(kind: BoundKind, sources: &BTreeMap<PathBuf, i32>) -> Html {
	html! {<div class="mx-2 mb-1">
		<span>
			<strong>{kind.display_name()}{". "}</strong>
			<span style="font-size: 14px;">
				{kind.description()}
			</span>
		</span>
		<table class="table table-compact table-striped mx-auto" style="width: 90%;">
			<thead>
				<tr class="text-center" style="color: var(--bs-heading-color);">
					<th scope="col" style="width: 50px;">{"Value"}</th>
					<th scope="col">{"Source"}</th>
				</tr>
			</thead>
			<tbody>
				{sources.iter().map(|(path, value)| html! {
					<tr>
						<td class="text-center">{value.to_string()}</td>
						<td>{crate::data::as_feature_path_text(path)}</td>
					</tr>
				}).collect::<Vec<_>>()}
			</tbody>
		</table>
	</div>}
}
