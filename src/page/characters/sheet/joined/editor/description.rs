use crate::{
	page::characters::sheet::{CharacterHandle, MutatorImpact},
	system::dnd5e::{
		components::{validate_uint_only, GeneralProp},
		data::character::{Persistent, PersonalityKind},
	},
	utility::InputExt,
};
use yew::prelude::*;

#[function_component]
pub fn DescriptionTab() -> Html {
	/* TODO: Age numerical field, with descriptive text for life expectancy. */
	html! {<div class="mx-4 mt-3">
		<SizeForm />
		<PersonalitySection />
		<AppearanceSection />
	</div>}
}

#[function_component]
fn SizeForm() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();

	let formula = state.derived_description().size_formula;
	let height = state.persistent().description.height;
	let weight = state.persistent().description.weight;
	let size = state.persistent().description.size();
	let size_info = size.description();
	let h_bonus_str = formula.height.bonus.as_nonzero_string();
	let w_mod_str = formula.weight.multiplier.as_nonzero_string();
	let height_range_str = format!("{} - {} inches", formula.min_height(), formula.max_height());
	let height_formula_str = format!(
		"{}{}",
		formula.height.base,
		h_bonus_str
			.as_ref()
			.map(|s| format!(" + {s} (modifier)"))
			.unwrap_or_default()
	);
	let weight_range_str = format!("{} - {} lbs", formula.min_weight(), formula.max_weight());
	let weight_formula_str = format!(
		"{}{}{}",
		formula.weight.base,
		h_bonus_str
			.zip(w_mod_str)
			.map(|(h, w)| format!(" + (height modifier ({h}) * {w})"))
			.unwrap_or_default(),
		formula
			.weight
			.bonus
			.as_nonzero_string()
			.map(|s| format!(" + {s}"))
			.unwrap_or_default()
	);
	let height_label = format!(
		"{ft}ft {ins}in ({cm}cm)",
		ft = height / 12,
		ins = height % 12,
		cm = ((height as f32) * 2.54).round() as u32
	);
	let weight_label = format!("{}kg", ((weight as f32) * 0.45359237).round() as u32);

	let set_height = state.new_dispatch(|evt: web_sys::Event, persistent| {
		let Some(value) = evt.input_value_t::<u32>() else { return MutatorImpact::None; };
		persistent.description.height = value;
		MutatorImpact::None
	});
	let set_weight = state.new_dispatch(|evt: web_sys::Event, persistent| {
		let Some(value) = evt.input_value_t::<u32>() else { return MutatorImpact::None; };
		persistent.description.weight = value;
		MutatorImpact::None
	});
	let roll_size = state.new_dispatch(move |_, persistent| {
		let mut rng = rand::thread_rng();
		let (height, weight) = formula.get_random(&mut rng);
		persistent.description.height = height;
		persistent.description.weight = weight;
		MutatorImpact::None
	});

	html! {
		<div class="mb-3">
			<h3>{"Size"}</h3>
			<div class="form-text mb-2" id="sizeHelp">
				<strong>{size}{": "}</strong>
				{size_info}
			</div>
			<div class="row g-0 mb-2">
				<div class="col me-1 d-flex align-items-center">
					<button class="btn btn-outline-success" onclick={roll_size}>{"Randomize using"}</button>
					<div class="input-group ms-2 w-auto flex-grow-1">
						<span class="input-group-text">{"Height"}</span>
						<span class="input-group-text">{height_range_str}</span>
						<span class="input-group-text bg-transparent flex-grow-1">{height_formula_str}</span>
					</div>
				</div>
				<div class="col ms-1">
					<div class="input-group">
						<span class="input-group-text">{"Weight"}</span>
						<span class="input-group-text">{weight_range_str}</span>
						<span class="input-group-text bg-transparent flex-grow-1">{weight_formula_str}</span>
					</div>
				</div>
			</div>
			<div class="row g-0">
				<div class="col me-1">
					<div class="input-group">
						<span class="input-group-text">{"Height (inches)"}</span>
						<input
							type="number" class="form-control text-center"
							id={"height"}
							min="0"
							value={format!("{height}")}
							onkeydown={validate_uint_only()}
							onchange={set_height}
						/>
						<span class="input-group-text">{height_label}</span>
					</div>
				</div>
				<div class="col ms-1">
					<div class="input-group">
						<span class="input-group-text">{"Weight (lbs)"}</span>
						<input
							type="number" class="form-control text-center"
							id={"weight"}
							min="0"
							value={format!("{weight}")}
							onkeydown={validate_uint_only()}
							onchange={set_weight}
						/>
						<span class="input-group-text">{weight_label}</span>
					</div>
				</div>
			</div>
		</div>
	}
}

#[function_component]
fn PersonalitySection() -> Html {
	html! {
		<div>
			<h3>{"Personality"}</h3>
			<PersonalityCard value={PersonalityKind::Trait} />
			<PersonalityCard value={PersonalityKind::Ideal} />
			<PersonalityCard value={PersonalityKind::Bond} />
			<PersonalityCard value={PersonalityKind::Flaw} />
		</div>
	}
}

#[function_component]
fn PersonalityCard(GeneralProp { value }: &GeneralProp<PersonalityKind>) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let personality_kind = *value;
	let add_item = state.new_dispatch(move |value, persistent| {
		persistent.description.personality[personality_kind].push(value);
		MutatorImpact::None
	});
	let info_collapse = {
		let collapse_id = format!("{}-info", value.to_string());
		html! {
			<div class="mb-2">
				<div class="mb-1">
					<button
						role="button" class={"collapse_trigger arrow_left collapsed"}
						data-bs-toggle="collapse"
						data-bs-target={format!("#{collapse_id}")}
					>
						{"Info"}
					</button>
				</div>
				<div class="collapse" id={collapse_id}>
					{value.description()}
				</div>
			</div>
		}
	};
	let selected_values = {
		let add_custom = add_item.reform(|_| String::new());
		let delete_item = state.new_dispatch(move |idx: usize, persistent| {
			persistent.description.personality[personality_kind].remove(idx);
			MutatorImpact::None
		});
		let update_item = state.new_dispatch(move |(idx, evt): (usize, web_sys::Event), persistent| {
			let Some(value) = evt.input_value() else { return MutatorImpact::None; };
			let Some(target) = persistent.description.personality[personality_kind].get_mut(idx) else { return MutatorImpact::None; };
			*target = value.trim().to_owned();
			MutatorImpact::None
		});
		let selected_traits = &state.persistent().description.personality[*value];
		html! {
			<div class="mb-3">
				<ul class="list-group mb-1">
					{selected_traits.iter().enumerate().map(|(idx, value)| {
						let on_delete = delete_item.reform(move |_| idx);
						let onchange = update_item.reform(move |evt| (idx, evt));
						html! {
							<li class="list-group-item d-flex p-0">
								<input
									type="text"
									class="form-control border-0 w-auto flex-grow-1 px-2"
									placeholder={format!("type your {personality_kind} here...")}
									value={value.clone()}
									{onchange}
								/>
								<button type="button" class="btn btn-danger btn-xs m-2" onclick={on_delete}>
									<i class="bi bi-trash me-1" />
									{"Delete"}
								</button>
							</li>
						}
					}).collect::<Vec<_>>()}
				</ul>
				<button role="button" class="btn btn-success btn-sm" onclick={add_custom}>
					<i class="bi bi-plus" />{"Add Custom"}
				</button>
			</div>
		}
	};
	let suggestions_collapsable = {
		let suggestions = &state.derived_description().personality_suggestions[*value];
		let collapse_id = format!("{}-suggestions", value.to_string());
		html! {
			<div>
				<div class="mb-2">
					<button
						role="button" class={"collapse_trigger arrow_left collapsed"}
						data-bs-toggle="collapse"
						data-bs-target={format!("#{collapse_id}")}
					>
						{"Suggestions"}
					</button>
				</div>
				<div class="collapse" id={collapse_id}>
					<ul class="list-group">
						{suggestions.iter().map(|value| {
							let onclick = add_item.reform({
								let value = value.clone();
								move |_| value.clone()
							});
							html! {
								<li class="list-group-item d-flex px-2">
									<button type="button" class="btn btn-outline-success btn-xs me-2" {onclick}>
										<i class="bi bi-plus" />
										{"Add"}
									</button>
									{value}
								</li>
							}
						}).collect::<Vec<_>>()}
					</ul>
				</div>
			</div>
		}
	};
	html! {
		<div class="mb-4">
			<h5>{value.to_string()}</h5>
			{selected_values}
			{info_collapse}
			{suggestions_collapsable}
		</div>
	}
}

#[function_component]
pub fn AppearanceSection() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let onchange = Callback::from({
		let state = state.clone();
		move |evt: web_sys::Event| {
			let Some(value) = evt.input_value() else { return; };
			state.dispatch(Box::new(move |persistent: &mut Persistent| {
				persistent.description.appearance = value;
				MutatorImpact::None
			}));
		}
	});
	html! {
		<div class="form-floating mb-3">
			<textarea
				class="form-control" id="appearance"
				{onchange}
				value={state.persistent().description.appearance.clone()}
			/>
			<label for="appearance">{"Appearance"}</label>
		</div>
	}
}
