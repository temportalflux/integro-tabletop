use std::cmp::Ordering;

use crate::{
	bootstrap::components::Tooltip,
	components::{modal, Tag, Tags},
	system::dnd5e::{
		components::SharedCharacter,
		data::{
			character::{HitPoint, Persistent},
			mutator::Defense,
		},
	},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

static TEXT_HIT_POINTS: &'static str = "\
Hit points represent a combination of physical and mental durability, \
the will to live, and luck. Creatures with more hit points are more \
difficult to kill. Those with fewer hit points are more fragile.

A creature's current hit points (usually just called hit points) \
can be any number from the creature's hit point maximum down to 0. \
This number changes frequently as a creature takes damage or receives healing.

Whenever a creature takes damage, that damage is subtracted from its hit points. \
The loss of hit points has no effect on a creature's capabilities \
until the creature drops to 0 hit points.";
static TEXT_TEMP_HP: &'static str = "\
Some spells and special abilities confer temporary hit points to a creature. \
Temporary hit points aren't actual hit points; they are a buffer against damage, \
a pool of hit points that protect you from injury.

When you have temporary hit points and take damage, the temporary hit points \
are lost first, and any leftover damage carries over to your normal hit points. \
For example, if you have 5 temporary hit points and take 7 damage, \
you lose the temporary hit points and then take 2 damage.

Because temporary hit points are separate from your actual hit points, \
they can exceed your hit point maximum. A character can, therefore, \
be at full hit points and receive temporary hit points.

Healing can't restore temporary hit points, and they can't be added together. \
If you have temporary hit points and receive more of them, you decide whether \
to keep the ones you have or to gain the new ones. For example, if a spell \
grants you 12 temporary hit points when you already have 10, \
you can have 12 or 10, not 22.

If you have 0 hit points, receiving temporary hit points doesn't restore you \
to consciousness or stabilize you. They can still absorb damage directed at \
you while you're in that state, but only true healing can save you.

Unless a feature that grants you temporary hit points has a duration, \
they last until they're depleted or you finish a long rest.";
static TEXT_HEALING: &'static str = "\
Unless it results in death, damage isn't permanent. Even death is reversible \
through powerful magic. Rest can restore a creature's hit points, \
and magical methods such as a cure wounds spell or a \
potion of healing can remove damage in an instant.

When a creature receives healing of any kind, hit points regained are added \
to its current hit points. A creature's hit points can't exceed its \
hit point maximum, so any hit points regained in excess of \
this number are lost. For example, a druid grants a ranger \
8 hit points of healing. If the ranger has 14 current hit points \
and has a hit point maximum of 20, the ranger regains 6 hit points from the druid, not 8.

A creature that has died can't regain hit points until magic \
such as the revivify spell has restored it to life.";
static TEXT_DROP_TO_ZERO: &'static str = "\
When you drop to 0 hit points, you either die outright or fall unconscious, \
as explained in the following sections.";
static TEXT_DTZ_INSTANT_DEATH: &'static str = "\
Massive damage can kill you instantly. When damage reduces you to 0 hit points and there is \
damage remaining, you die if the remaining damage equals or exceeds your hit point maximum.

For example, a cleric with a maximum of 12 hit points currently has 6 hit points. \
If she takes 18 damage from an attack, she is reduced to 0 hit points, but 12 damage remains. \
Because the remaining damage equals her hit point maximum, the cleric dies.";
static TEXT_DTZ_FALLING_UNCONSCIOUS: &'static str = "\
If damage reduces you to 0 hit points and fails to kill you, you fall unconscious. \
This unconsciousness ends if you regain any hit points.";
static TEXT_DTZ_SAVING_THROWS: &'static str = "\
Whenever you start your turn with 0 hit points, you must make a special saving throw, \
called a death saving throw, to determine whether you creep closer to death or hang onto life. \
Unlike other saving throws, this one isn't tied to any ability score. \
You are in the hands of fate now, aided only by spells and features that improve your \
chances of succeeding on a saving throw.";
static TEXT_DTZ_SAVING_THROWS_ROLL: &'static str = "\
If the roll is 10 or higher, you succeed. Otherwise, you fail. \
A success or failure has no effect by itself. On your third success, \
you become stable (see below). On your third failure, you die. The successes and failures \
don't need to be consecutive; keep track of both until you collect three of a kind. \
The number of both is reset to zero when you regain any hit points or become stable.";
static TEXT_DTZ_SAVING_THROWS_ROLL_CRIT: &'static str = "\
When you make a death saving throw and roll a 1 on the d20, it counts as two failures. \
If you roll a 20 on the d20, you regain 1 hit point.";
static TEXT_DTZ_SAVING_THROWS_DMG: &'static str = "\
If you take any damage while you have 0 hit points, you suffer a death saving throw failure. \
If the damage is from a critical hit, you suffer two failures instead. If the damage equals \
or exceeds your hit point maximum, you suffer instant death.";
static TEXT_DTZ_STABILIZING: &'static str = "\
The best way to save a creature with 0 hit points is to heal it. If healing is unavailable, \
the creature can at least be stabilized so that it isn't killed by a failed death saving throw.

You can use your action to administer first aid to an unconscious creature and \
attempt to stabilize it, which requires a successful DC 10 Wisdom (Medicine) check.

A stable creature doesn't make death saving throws, even though it has 0 hit points, \
but it does remain unconscious. The creature stops being stable, and must start making \
death saving throws again, if it takes any damage. A stable creature that isn't \
healed regains 1 hit point after 1d4 hours.";

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
			character.add_assign_hit_points(amt as i32, prev.hit_points(HitPoint::Max));
		}
	});
	let onclick_dmg = state.new_dispatch({
		let take_hp_input = take_hp_input.clone();
		move |evt: MouseEvent, character, prev| {
			evt.stop_propagation();
			let Some(amt) = take_hp_input.emit(()) else { return; };
			character.add_assign_hit_points(-1 * (amt as i32), prev.hit_points(HitPoint::Max));
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

	let max_hp_table = {
		let rows = state.max_hit_points().sources().iter().fold(Vec::new(), |mut html, (source, bonus)| {
			html.push(html! {
				<tr>
					<td class="text-center">{*bonus}</td>
					<td>{crate::data::as_feature_path_text(source).unwrap_or_default()}</td>
				</tr>
			});
			html
		});
		html! {
			<table class="table table-compact table-striped m-0">
				<thead>
					<tr class="text-center" style="color: var(--bs-heading-color);">
						<th scope="col">{"Bonus"}</th>
						<th scope="col">{"Source"}</th>
					</tr>
				</thead>
				<tbody>
					{rows}
				</tbody>
			</table>
		}
	};

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

	let delta = use_state_eq(|| 0i32);
	let (delta_sig, delta_abs) = (delta.signum(), delta.abs() as u32);
	let prev_hp = state.hit_points(HitPoint::Current);
	let prev_temp = state.hit_points(HitPoint::Temp);
	let (next_hp, next_temp) = state
		.persistent()
		.add_hit_points(*delta, state.hit_points(HitPoint::Max));
	let healing_amt = delta_sig.max(0) as u32 * delta_abs;
	let damage_amt = (-delta_sig).max(0) as u32 * delta_abs;
	let new_hp_color_classes = match next_hp.cmp(&prev_hp) {
		Ordering::Greater => classes!("heal"),
		Ordering::Less => classes!("damage"),
		Ordering::Equal => classes!(),
	};
	let temp_hp_color_classes = match next_temp.cmp(&prev_temp) {
		Ordering::Greater => classes!("heal"),
		Ordering::Less => classes!("damage"),
		Ordering::Equal => classes!(),
	};
	let temp_hp_classes = (prev_temp <= 0)
		.then(|| classes!("d-none"))
		.unwrap_or_default();

	let onchange_healing = Callback::from({
		let delta = delta.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let Ok(value) = input.value().parse::<u32>() else { return; };
			delta.set(value as i32);
		}
	});
	let onchange_damage = Callback::from({
		let delta = delta.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			let Ok(value) = input.value().parse::<u32>() else { return; };
			delta.set(value as i32 * -1);
		}
	});
	let onclick_add = Callback::from({
		let delta = delta.clone();
		move |_| {
			delta.set(delta.saturating_add(1));
		}
	});
	let onclick_sub = Callback::from({
		let delta = delta.clone();
		move |_| {
			delta.set(delta.saturating_sub(1));
		}
	});
	let apply_delta = state.new_dispatch({
		let delta = delta.clone();
		move |_: MouseEvent, character, prev| {
			character.add_assign_hit_points(*delta, prev.hit_points(HitPoint::Max));
			delta.set(0);
		}
	});
	let clear_delta = Callback::from({
		let delta = delta.clone();
		move |_| {
			delta.set(0);
		}
	});

	// TODO: Show sources for how Max HP was calculated

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
								min="0" value={healing_amt.to_string()}
								onkeydown={validate_uint_only()}
								onchange={onchange_healing}
							/>
						</div>
					</div>

					<div class="d-flex justify-content-center">
						<button type="button" class="btn btn-theme hp-action sub" onclick={onclick_sub} />
						<button type="button" class="btn btn-theme hp-action add" onclick={onclick_add} />
					</div>

					<div class="row mx-0 my-2">
						<div class="col-4 p-0">
							<label
								class={classes!(
									"col-form-label",
									"text-center",
									"theme-damage"
								)}
								for="inputDamage" style="width: 100%"
							>{"Damage"}</label>
						</div>
						<div class="col">
							<input
								class={classes!(
									"form-control",
									"text-center",
									"theme-damage"
								)}
								type="number" id="inputDamage"
								style="font-size: 20px; font-weight: 500; padding: 0; height: 100%;"
								min="0" value={damage_amt.to_string()}
								onkeydown={validate_uint_only()}
								onchange={onchange_damage}
							/>
						</div>
					</div>

				</div>
				<div class="col-auto text-center m-auto">

					<div class="row m-0">
						<div class={{
							let mut classes = classes!("col");
							classes.extend(new_hp_color_classes.clone());
							classes
						}}>
							<h6 class="m-0 new-hp-header">{"NEW HP"}</h6>
							<div style="font-size: 40px; font-weight: 500; margin-top: -10px;">{next_hp}</div>
						</div>
						<div class={{
							let mut classes = classes!("col");
							classes.extend(temp_hp_color_classes.clone());
							classes.extend(temp_hp_classes);
							classes
						}}>
							<h6 class="m-0 new-hp-header">{"TEMP HP"}</h6>
							<div style="font-size: 40px; font-weight: 500; margin-top: -10px;">{next_temp}</div>
						</div>
					</div>

					<button
						type="button"
						class="m-2 btn btn-theme"
						disabled={*delta == 0}
						onclick={apply_delta}
					>{"Apply Changes"}</button>
					<button
						type="button"
						class="m-2 btn btn-outline-theme"
						disabled={*delta == 0}
						onclick={clear_delta}
					>{"Cancel"}</button>
				</div>
			</div>
			<span class="my-3" style="display: block; width: 100%; border-style: solid; border-width: 0; border-bottom-width: var(--bs-border-width); border-color: var(--theme-frame-color-muted);" />
			<div class="accordion" id="hitPointsInformation">
				<div class="accordion-item">
					<h2 class="accordion-header">
						<button
							class="accordion-button collapsed" type="button"
							data-bs-toggle="collapse" data-bs-target="#collapseMaxHP"
						>{"Max HP Breakdown"}</button>
					</h2>
					<div id="collapseMaxHP" class="accordion-collapse collapse" data-bs-parent="#hitPointsInformation">
						<div class="accordion-body" style="white-space: pre-line;">
							{max_hp_table}
						</div>
					</div>
				</div>
				<div class="accordion-item">
					<h2 class="accordion-header">
						<button
							class="accordion-button collapsed" type="button"
							data-bs-toggle="collapse" data-bs-target="#collapseHitPoints"
						>{"Hit Points"}</button>
					</h2>
					<div id="collapseHitPoints" class="accordion-collapse collapse" data-bs-parent="#hitPointsInformation">
						<div class="accordion-body" style="white-space: pre-line;">
							{TEXT_HIT_POINTS}
						</div>
					</div>
				</div>
				<div class="accordion-item">
					<h2 class="accordion-header">
						<button
							class="accordion-button collapsed" type="button"
							data-bs-toggle="collapse" data-bs-target="#collapseTempHP"
						>{"Temporary Hit Points"}</button>
					</h2>
					<div id="collapseTempHP" class="accordion-collapse collapse" data-bs-parent="#hitPointsInformation">
						<div class="accordion-body" style="white-space: pre-line;">
							{TEXT_TEMP_HP}
						</div>
					</div>
				</div>
				<div class="accordion-item">
					<h2 class="accordion-header">
						<button
							class="accordion-button collapsed" type="button"
							data-bs-toggle="collapse" data-bs-target="#collapseHealing"
						>{"Healing"}</button>
					</h2>
					<div id="collapseHealing" class="accordion-collapse collapse" data-bs-parent="#hitPointsInformation">
						<div class="accordion-body" style="white-space: pre-line;">
							{TEXT_HEALING}
						</div>
					</div>
				</div>
				<div class="accordion-item">
					<h2 class="accordion-header">
						<button
							class="accordion-button collapsed" type="button"
							data-bs-toggle="collapse" data-bs-target="#collapseDTZ"
						>{"Dropping to 0 Hit Points"}</button>
					</h2>
					<div id="collapseDTZ" class="accordion-collapse collapse" data-bs-parent="#hitPointsInformation">
						<div class="accordion-body" style="white-space: pre-line;">
							{TEXT_DROP_TO_ZERO}
							<span class="d-block my-2" />
							<div class="accordion" id="drop-to-zero">
								<div class="accordion-item">
									<h2 class="accordion-header">
										<button
											class="accordion-button collapsed" type="button"
											data-bs-toggle="collapse" data-bs-target="#collapseDTZInstantDeath"
										>{"Instant Death"}</button>
									</h2>
									<div id="collapseDTZInstantDeath" class="accordion-collapse collapse" data-bs-parent="#drop-to-zero">
										<div class="accordion-body" style="white-space: pre-line;">
											{TEXT_DTZ_INSTANT_DEATH}
										</div>
									</div>
								</div>
								<div class="accordion-item">
									<h2 class="accordion-header">
										<button
											class="accordion-button collapsed" type="button"
											data-bs-toggle="collapse" data-bs-target="#collapseDTZUnconscious"
										>{"Falling Unconscious"}</button>
									</h2>
									<div id="collapseDTZUnconscious" class="accordion-collapse collapse" data-bs-parent="#drop-to-zero">
										<div class="accordion-body" style="white-space: pre-line;">
											{TEXT_DTZ_FALLING_UNCONSCIOUS}
										</div>
									</div>
								</div>
								<div class="accordion-item">
									<h2 class="accordion-header">
										<button
											class="accordion-button collapsed" type="button"
											data-bs-toggle="collapse" data-bs-target="#collapseDTZSavingThrows"
										>{"Death Saving Throws"}</button>
									</h2>
									<div id="collapseDTZSavingThrows" class="accordion-collapse collapse" data-bs-parent="#drop-to-zero">
										<div class="accordion-body" style="white-space: pre-line;">
											{TEXT_DTZ_SAVING_THROWS}
											<br /><br />
											<strong>{"Roll a d20. "}</strong>
											{TEXT_DTZ_SAVING_THROWS_ROLL}
											<br /><br />
											<strong>{"Rolling 1 or 20. "}</strong>
											{TEXT_DTZ_SAVING_THROWS_ROLL_CRIT}
											<br /><br />
											<strong>{"Damage at 0 Hit Points. "}</strong>
											{TEXT_DTZ_SAVING_THROWS_DMG}
										</div>
									</div>
								</div>
								<div class="accordion-item">
									<h2 class="accordion-header">
										<button
											class="accordion-button collapsed" type="button"
											data-bs-toggle="collapse" data-bs-target="#collapseDTZStabilizing"
										>{"Stabilizing a Creature"}</button>
									</h2>
									<div id="collapseDTZStabilizing" class="accordion-collapse collapse" data-bs-parent="#drop-to-zero">
										<div class="accordion-body" style="white-space: pre-line;">
											{TEXT_DTZ_STABILIZING}
										</div>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>
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
