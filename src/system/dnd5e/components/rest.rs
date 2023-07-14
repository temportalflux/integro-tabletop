use std::path::PathBuf;

use super::GeneralProp;
use crate::components::modal;
use crate::page::characters::sheet::CharacterHandle;
use crate::system::dnd5e::components::{ConsumedUsesInput, validate_uint_only};
use crate::system::dnd5e::data::roll::Die;
use crate::system::dnd5e::data::Ability;
use crate::system::dnd5e::{components::glyph::Glyph, data::Rest};
use crate::utility::InputExt;
use yew::prelude::*;

#[function_component]
pub fn Button(GeneralProp { value }: &GeneralProp<Rest>) -> Html {
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let onclick = modal_dispatcher.callback({
		let rest = *value;
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("rest"),
				content: html! {<Modal value={rest} />},
				..Default::default()
			})
		}
	});

	let glyph_classes = classes!("rest", value.to_string().to_lowercase(), "me-1");
	html! {
		<button class="btn btn-outline-theme btn-sm me-3" {onclick}>
			<Glyph classes={glyph_classes} />
			{value.to_string()}{" Rest"}
		</button>
	}
}

#[function_component]
fn Modal(GeneralProp { value }: &GeneralProp<Rest>) -> Html {
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{value}{" Rest"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div class="text-block">{value.description()}</div>
			<HitDiceByClass />
		</div>
	</>}
}

#[function_component]
fn HitDiceByClass() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let mut sections = Vec::new();
	for class in &state.persistent().classes {
		sections.push(html! {
			<ClassHitDice
				name={class.name.clone()}
				die={class.hit_die}
				max_uses={class.current_level as u32}
				path={class.hit_die_selector.get_data_path()}
			/>
		});
	}
	html!(<>{sections}</>)
}

#[derive(Clone, PartialEq, Properties)]
struct ClassHitDiceProps {
	name: AttrValue,
	die: Die,
	max_uses: u32,
	path: Option<PathBuf>,
}
#[function_component]
fn ClassHitDice(
	ClassHitDiceProps {
		name,
		die,
		max_uses,
		path,
	}: &ClassHitDiceProps,
) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let uses_to_consume = use_state_eq(|| 0u32);
	let rolled_hp = use_state_eq(|| 0i32);
	let bonus_per_use = state.ability_modifier(Ability::Constitution, None);
	let consumed_uses = path
		.as_ref()
		.map(|path| state.get_first_selection_at::<u32>(path));
	let consumed_uses = consumed_uses
		.flatten()
		.map(Result::ok)
		.flatten()
		.unwrap_or(0);
	let set_consumed_uses_delta = Callback::from({
		let uses_to_consume = uses_to_consume.clone();
		move |delta: i32| {
			uses_to_consume.set(delta as u32);
		}
	});
	let set_rolled_hp = Callback::from({
		let rolled_hp = rolled_hp.clone();
		move |evt: web_sys::Event| {
			let Some(value) = evt.input_value_t::<u32>() else { return; };
			rolled_hp.set(value as i32);
		}
	});
	html! {
		<div class="mt-2">
			<h4>{name}</h4>
			<span>
				{"Gain 1"}{die.to_string()}
				{(bonus_per_use != 0).then(|| format!("{bonus_per_use:+}"))}
				{" hit points per use."}
			</span>
			<div class="me-2">{"Hit Dice available (resets on Long Rest):"}</div>
			<div class="uses d-flex justify-content-center">
				<ConsumedUsesInput
					{max_uses} {consumed_uses}
					data_path={path.clone()}
					can_remove_consumed_uses={false}
					consume_delta={set_consumed_uses_delta}
				/>
			</div>
			{(*uses_to_consume > 0).then(|| {
				html! {<>
					<div class="d-flex align-items-center justify-content-center">
						<span>
							{"Roll "}{*uses_to_consume}{die.to_string()}{" & input the sum:"}
						</span>
						<input
							type="number" class="form-control text-center ms-3"
							style="font-size: 20px; padding: 0; height: 30px; width: 60px;"
							min="0"
							value={format!("{}", *rolled_hp)}
							onkeydown={validate_uint_only()}
							onchange={set_rolled_hp}
						/>
					</div>
					<span class="text-block">
						{format!(
							"You will gain {} hit points.\n({} rolled HP + {:+} constitution modifier)",
							(*rolled_hp + bonus_per_use).max(0),
							*rolled_hp, bonus_per_use
						)}
					</span>
				</>}
			}).unwrap_or_default()}
		</div>
	}
}
