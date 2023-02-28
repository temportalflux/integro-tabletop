use crate::{
	bootstrap::components::Tooltip,
	system::dnd5e::{
		components::SharedCharacter,
		data::{character::{HitPoint, Persistent}, mutator::Defense},
	}, components::{Tags, Tag, modal},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

static TEXT: &'static str = "\
TODO: HP rules description
";

fn defence_to_html(defence: Defense) -> Html {
	let style = "width: 12px; height: 12px;".to_owned();
	match defence {
		Defense::Resistant => html! {
			<svg {style} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#00c680" d="M21.18969,15.5h-4.12v7.44h4.12a3.68142,3.68142,0,0,0,2.79-.97,3.75732,3.75732,0,0,0,.94-2.73,3.81933,3.81933,0,0,0-.95-2.74A3.638,3.638,0,0,0,21.18969,15.5Z"></path>
				<path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-8.11,29.51h-6.97l-4.77-9.56h-3.53v9.56h-6.51V10.49h10.63c3.2,0,5.71.71,7.51,2.13a7.21618,7.21618,0,0,1,2.71,6.03,8.78153,8.78153,0,0,1-1.14,4.67005,8.14932,8.14932,0,0,1-3.57,3l5.64,10.91Z"></path>
			</svg>
		},
		Defense::Immune => html! {
			<svg {style} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-16.75,29.42h-6.5V10.4h6.5Z"></path>
			</svg>
		},
		Defense::Vulnerable => html! {
			<svg {style} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#e40712" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-16.63,30.42h-7.12l-9.02-27.02h7.22L20.2597,31.07l5.38-19.67h7.27Z"></path>
			</svg>
		},
	}
}

#[function_component]
pub fn HitPoints() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let hp_input_node = use_node_ref();
	let take_hp_input = Callback::from({
		let node = hp_input_node.clone();
		move |_: ()| {
			let Some(node) = node.get() else { return None; };
			let Some(input) = node.dyn_ref::<HtmlInputElement>() else { return None; };
			let value = input.value();
			if value.is_empty() {
				return Some(1);
			}
			let Ok(value) = value.parse::<u32>() else { return None; };
			input.set_value("");
			Some(value)
		}
	});
	let onclick_heal = state.new_dispatch({
		let take_hp_input = take_hp_input.clone();
		move |evt: MouseEvent, character, prev| {
			evt.stop_propagation();
			let Some(amt) = take_hp_input.emit(()) else { return; };
			character.add_hp(amt, prev.hit_points(HitPoint::Max));
		}
	});
	let onclick_dmg = state.new_dispatch({
		let take_hp_input = take_hp_input.clone();
		move |evt: MouseEvent, character, _| {
			evt.stop_propagation();
			let Some(amt) = take_hp_input.emit(()) else { return; };
			character.sub_hp(amt);
		}
	});
	let onclick_amt = Callback::from(|evt: MouseEvent| evt.stop_propagation());
	let onclick = modal_dispatcher.callback(|_| {
		modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("hit-points"),
				content: html! {<Modal />},
				..Default::default()
		})
	});

	html! {
		<div class="card m-1" style="height: 80px;">
			<div class="card-body" style="padding: 5px 5px;">
				<div class="d-flex">
					<div class="flex-grow-1" {onclick}>
						<h5 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color); margin: 0 0 2px 0;">{"Hit Points"}</h5>
						<div class="row text-center m-0" style="--bs-gutter-x: 0;">
							<div class="col" style="min-width: 50px;">
								<div style="font-size: 0.75rem; padding: 0 5px;">{"Current"}</div>
								<div style="font-size: 26px; font-weight: 500;">{state.hit_points(HitPoint::Current)}</div>
							</div>
							<div class="col-auto">
								<div style="min-height: 1.2rem;"></div>
								<div style="font-size: 23px; font-weight: 300;">{"/"}</div>
							</div>
							<div class="col" style="min-width: 50px;">
								<div style="font-size: 0.75rem; padding: 0 5px;">{"Max"}</div>
								<div style="font-size: 26px; font-weight: 500;">{state.hit_points(HitPoint::Max)}</div>
							</div>
							<div class="col" style="min-width: 50px; margin: 0 5px;">
								<div style="font-size: 0.75rem;">{"Temp"}</div>
								<div style="font-size: 26px; font-weight: 300;">{state.hit_points(HitPoint::Temp)}</div>
							</div>
						</div>
					</div>
					<div style="width: 80px;">
						<button
							type="button" class="btn btn-success"
							style="vertical-align: top; width: 100%; --bs-btn-padding-y: 0px; --bs-btn-font-size: .75rem;"
							onclick={onclick_heal}
						>{"Heal"}</button>
						<input ref={hp_input_node}
							type="number" class="form-control text-center" id="hp-amount"
							style="padding: 0; margin: 0 0 4px 0; height: 20px;"
							min="0"
							onclick={onclick_amt} onkeydown={validate_uint_only()}
						/>
						<button
							type="button" class="btn btn-danger"
							style="vertical-align: top; width: 100%; --bs-btn-padding-y: 0px; --bs-btn-font-size: .75rem;"
							onclick={onclick_dmg}
						>{"Damage"}</button>
					</div>
				</div>
			</div>
		</div>
	}
}

fn validate_uint_only() -> Callback<KeyboardEvent> {
	Callback::from(|evt: KeyboardEvent| {
		if !evt.cancelable() {
			log::error!("Cannot cancel input::onkeydown event");
			return;
		}
		if evt.key().len() == 1 && evt.key().parse::<u32>().is_err() {
			evt.prevent_default();
		}
	})
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	let temp_hp_oncommit = Callback::from({
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let Ok(value) = input.value().parse::<u32>() else { return; };
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				*persistent.temp_hp_mut() = value;
			}));
		}
	});

	/* TODO: HP Form
	New HP: label + value
	- white when matching current hp
	- red when <
	- green when >
	- displays the projected post-adjustment value
	- TODO: how does this interact with temp hp
	Healing (input)
	- green border + label
	- directly modifying clears damage (input) on focus lost
	Damage (input)
	- red border + label
	- directly modifying clears healing (input) on focus lost
	+ Button: +1 to diff
	- Button: -1 to diff
	Apply Changes Button: Commits the diff to character hp (disabled/hidden if new hp = current hp)
	Cancel Button: Discards changes to form (disabled/hidden if new hp = current hp)
	*/
	
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Hit Point Management"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div class="row my-1" style="--bs-gutter-x: 0;">
				<div class="col text-center">
					<h6>{"CURRENT HP"}</h6>
					<div style="font-size: 26px; font-weight: 500;">{state.hit_points(HitPoint::Current)}</div>
				</div>
				<div class="col text-center">
					<h6>{"MAX HP"}</h6>
					<div style="font-size: 26px; font-weight: 500;">{state.hit_points(HitPoint::Max)}</div>
				</div>
				<div class="col text-center">
					<h6>{"TEMP HP"}</h6>
					<input
						type="number" class="form-control text-center"
						style="font-size: 26px; font-weight: 500; padding: 0; height: 40px;"
						min="0"
						value={format!("{}", state.hit_points(HitPoint::Temp))}
						onkeydown={validate_uint_only()}
						onchange={temp_hp_oncommit}
					/>
				</div>
			</div>
			<span class="my-3" style="display: block; width: 100%; border-style: solid; border-width: 0; border-bottom-width: var(--bs-border-width); border-color: var(--theme-frame-color-muted);" />
			<div class="row my-1">
				<div class="col">

					<div class="row mx-0 my-2">
						<div class="col-4 p-0">
							<label class="col-form-label text-center theme-healing" for="inputHealing" style="width: 100%">{"Healing"}</label>
						</div>
						<div class="col">
							<input
								class="form-control text-center theme-healing"
								type="number" id="inputHealing"
								style="font-size: 20px; font-weight: 500; padding: 0; height: 100%;"
								min="0" value="0"
							/>
						</div>
					</div>
					
					<div class="d-flex justify-content-center">
						<button type="button" class="btn btn-theme hp-action sub" />
						<button type="button" class="btn btn-theme hp-action add" />
					</div>

					<div class="row mx-0 my-2">
						<div class="col-4 p-0">
							<label class="col-form-label text-center theme-damage" for="inputDamage" style="width: 100%">{"Damage"}</label>
						</div>
						<div class="col">
							<input
								class="form-control text-center theme-damage"
								type="number" id="inputDamage"
								style="font-size: 20px; font-weight: 500; padding: 0; height: 100%;"
								min="0" value="0"
							/>
						</div>
					</div>
					
				</div>
				<div class="col-auto text-center m-auto">
					<h6 class="m-0">{"NEW HP"}</h6>
					<div style="font-size: 40px; font-weight: 500; margin-top: -10px;">{state.hit_points(HitPoint::Current)}</div>
					<button type="button" class="m-2 btn btn-theme">{"Apply Changes"}</button>
					<button type="button" class="m-2 btn btn-outline-theme">{"Cancel"}</button>
				</div>
			</div>
			<span class="my-3" style="display: block; width: 100%; border-style: solid; border-width: 0; border-bottom-width: var(--bs-border-width); border-color: var(--theme-frame-color-muted);" />
			<div class="my-1" style="white-space: pre-line;">
				{TEXT}
			</div>
		</div>
	</>}
}

#[function_component]
pub fn DefensesCard() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let defenses = state
		.defenses()
		.iter()
		.fold(Vec::new(), |all, (kind, targets)| {
			targets.iter().fold(all, |mut all, (target, sources)| {
				let tooltip = crate::data::as_feature_paths_html(sources.iter());
				all.push(html! {
					<Tooltip tag={"span"} style={"margin: 2px;"} content={tooltip} use_html={true}>
						<Tag>
							{defence_to_html(kind)}
							<span style="margin-left: 5px;">{target.clone()}</span>
						</Tag>
					</Tooltip>
				});
				all
			})
		});
	html! {
		<div class="card m-1" style="height: 100px;">
			<div class="card-body text-center" style="padding: 5px 5px;">
				<h6 class="card-title" style="font-size: 0.8rem;">{"Defenses"}</h6>
				<div class="d-flex justify-content-center" style="overflow: hidden;">
					{match defenses.is_empty() {
						true => html! { "None" },
						false => html! {<Tags> {defenses} </Tags>},
					}}
				</div>
			</div>
		</div>
	}
}

#[function_component]
pub fn ConditionsCard() -> Html {
	html! {
		<div class="card m-1" style="height: 100px;">
			<div class="card-body text-center" style="padding: 5px 5px;">
				<h6 class="card-title" style="font-size: 0.8rem;">{"Conditions"}</h6>
				
			</div>
		</div>
	}
}
