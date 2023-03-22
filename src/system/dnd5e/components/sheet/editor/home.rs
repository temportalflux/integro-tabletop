use crate::system::dnd5e::{components::SharedCharacter, data::character::Persistent};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn HomeTab() -> Html {
	// TODO: In here is where module selection will go.
	// Modules enabled for the character is a subset of the modules the user has access to.
	// Only modules enabled for the character are available in the editor/viewer.

	// More robust pronoun selection? https://twitter.com/Patch_Games/status/1423706763841347586
	html! {<div class="mx-4">
		<div class="my-3">
			<h4>{"Character Info"}</h4>
			<p>{"Who is your character? These options are also available in the Description tab."}</p>
			<div class="row">
				<div class="col-5">
					<NameEditor />
				</div>
				<div class="col">
					<PronounEditor />
				</div>
			</div>
		</div>
		<div class="my-3">
			<SettingsEditor />
		</div>
	</div>}
}

#[function_component]
fn NameEditor() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let onchange = Callback::from({
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let value = input.value();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				persistent.description.name = value;
				None
			}));
		}
	});
	html! {<>
		<label for="nameInput" class="form-label">{"Name"}</label>
		<input  id="nameInput" class="form-control" type="text"
			value={state.persistent().description.name.clone()}
			{onchange}
		/>
	</>}
}

#[function_component]
fn PronounEditor() -> Html {
	static PROVIDED_OPTIONS: [(&'static str, &'static str); 3] = [
		("she/her", "She / Her"),
		("he/him", "He / Him"),
		("they/them", "They / Them"),
	];
	let state = use_context::<SharedCharacter>().unwrap();
	let onchange = Callback::from({
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let is_checkbox = input.type_() == "checkbox";
			let is_checked = input.checked();
			let value = input.value();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				match (is_checkbox, is_checked) {
					(true, true) => {
						persistent.description.pronouns.insert(value);
					}
					(true, false) => {
						persistent.description.pronouns.remove(&value);
					}
					(false, _) => {
						persistent.description.custom_pronouns = value.trim().to_owned();
					}
				}
				None
			}));
		}
	});
	html! {
		<div class="pronouns-group">
			<label for="pronouns" class="form-label">{"Pronouns"}</label>
			<div class="input-group">
				{PROVIDED_OPTIONS.iter().map(|(value, label)| html! {
					<div class="input-group-text">
						<label for={value.to_owned()} class="form-check-label me-2">{label.to_owned()}</label>
						<input  id={value.to_owned()}
							class="form-check-input mt-0 success" type="checkbox"
							value={value.to_owned()}
							checked={state.persistent().description.pronouns.contains(*value)}
							onchange={onchange.clone()}
						/>
					</div>
				}).collect::<Vec<_>>()}
				<input id="pronouns"
					class="form-control" type="text"
					placeholder="and/or add custom pronouns"
					onchange={onchange.clone()}
					value={state.persistent().description.custom_pronouns.clone()}
				/>
			</div>
		</div>
	}
}

#[function_component]
fn SettingsEditor() -> Html {
	html! {<>
		<h4>{"Settings"}</h4>
		<AutoExchangeSwitch />
	</>}
}

#[function_component]
fn AutoExchangeSwitch() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let onchange = Callback::from({
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let value = input.checked();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				persistent.settings.currency_auto_exchange = value;
				None
			}));
		}
	});
	html! {
		<div class="form-check form-switch">
			<input
				class="form-check-input"
				type="checkbox" role="switch" id="auto_exchange"
				aria-describedby="auto_exchange-help"
				onchange={onchange}
				checked={state.persistent().settings.currency_auto_exchange}
			/>
			<label class="form-check-label" for="auto_exchange">
				<strong>{"Currency: "}</strong>
				{"Auto-Exchange"}
			</label>
			<div id="auto_exchange-help" class="form-text text-block">
				{"If enabled, coinage will be automatically exchanged for smaller kinds when removing coinage.
				The exchange button also becomes available, which allows you to convert smaller coinage \
				into the largest possible coins."}
			</div>
		</div>
	}
}
