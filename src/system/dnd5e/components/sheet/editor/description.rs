use crate::{system::dnd5e::{
	components::{validate_uint_only, SharedCharacter},
	data::Size,
}, utility::InputExt};
use yew::prelude::*;

#[function_component]
pub fn DescriptionTab() -> Html {
	/*
	Age numerical field, with descriptive text for life expectancy.
	Personality Traits, Ideals, Bonds, Flaws
	*/


	html! {<div class="mx-4 mt-3">
		<SizeForm />
		<div class="form-floating mb-3">
			<textarea class="form-control" id="appearance" />
			<label for="appearance">{"Appearance"}</label>
		</div>
	</div>}
}

#[function_component]
fn SizeForm() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	let formula = state.derived_description().size_formula;
	let height = state.persistent().description.height;
	let weight = state.persistent().description.weight;
	let size = state.persistent().description.size();
	let size_info = match size {
		Size::Small => "Creatures less than 45 inches tall are Small sized. You control a 5 by 5 ft. space in combat. You can squeeze through Tiny spaces.",
		Size::Medium => "Creatures at least 45 inches tall are Medium sized. You control a 5 by 5 ft. space in combat. You can squeeze through Small spaces.",
		_ => "",
	};
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

	let set_height = state.new_dispatch(|evt: web_sys::Event, persistent, _| {
		let Some(value) = evt.input_value_t::<u32>() else { return None; };
		persistent.description.height = value;
		None
	});
	let set_weight = state.new_dispatch(|evt: web_sys::Event, persistent, _| {
		let Some(value) = evt.input_value_t::<u32>() else { return None; };
		persistent.description.weight = value;
		None
	});
	let roll_size = state.new_dispatch(|_, persistent, character| {
		let mut rng = rand::thread_rng();
		let (height, weight) = character.derived_description().size_formula.get_random(&mut rng);
		persistent.description.height = height;
		persistent.description.weight = weight;
		None
	});

	html! {
		<div class="mb-3">
			<label>{"Size"}</label>
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
