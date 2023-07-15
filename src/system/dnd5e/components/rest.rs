use super::GeneralProp;
use crate::{
	components::{modal, stop_propagation},
	page::characters::sheet::{CharacterHandle, MutatorImpact},
	system::dnd5e::{
		components::{glyph::Glyph, validate_uint_only},
		data::{
			roll::{Die, Roll, RollSet},
			Ability, Rest,
		},
	},
	utility::InputExt,
};
use enum_map::EnumMap;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
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
	let state = use_context::<CharacterHandle>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let hit_dice_to_consume = use_state_eq(|| HitDiceToConsume::default());

	let constitution_mod = state.ability_modifier(Ability::Constitution, None);
	let commit_rest = state.new_dispatch({
		let rest = *value;
		let hit_dice_to_consume = hit_dice_to_consume.clone();
		let max_hp = state.max_hit_points().value();
		let resets = state.rest_resets().get(*value).clone();
		let close_modal = modal_dispatcher.callback(|_| modal::Action::Close);
		move |_, persistent| {
			let mut rng = rand::thread_rng();
			let mut changes = Vec::new();
			match rest {
				Rest::Long => {
					persistent.hit_points.current = max_hp;
					changes.push(format!("Set hit points to max ({max_hp})."));

					persistent.hit_points.temp = 0;
					changes.push(format!("Cleared temporary hit points."));

					persistent.hit_points.failure_saves = 0;
					persistent.hit_points.success_saves = 0;
					changes.push(format!("Cleared saving throws."));

					let hit_die_paths = persistent
						.classes
						.iter()
						.filter_map(|class| class.hit_die_selector.get_data_path())
						.collect::<Vec<_>>();
					for data_path in hit_die_paths {
						persistent.set_selected(data_path, None);
					}
					changes.push(format!("Restored all hit dice."));
				}
				Rest::Short => {
					let hp = hit_dice_to_consume.hp_to_gain(constitution_mod);
					persistent.hit_points.current += hp;
					changes.push(format!("Increased hit points by {hp}."));

					for (amount_used, data_path) in hit_dice_to_consume.consumed_uses() {
						let prev_value = persistent.get_first_selection_at::<u32>(data_path);
						let prev_value = prev_value.map(Result::ok).flatten().unwrap_or(0);
						let new_value = prev_value.saturating_add(*amount_used).to_string();
						persistent.set_selected(data_path, Some(new_value));

						let class_name = data_path.components().next().unwrap().as_os_str();
						let class_name = class_name.to_str().unwrap();
						changes.push(format!("Used {amount_used} hit dice from {class_name}."));
					}
				}
			};
			for entry in &resets {
				let uses_to_remove = match &entry.restore_amount {
					None => None,
					Some(roll) => Some(roll.roll(&mut rng)),
				};

				let path_str = entry.source.display().to_string();
				let path_str = path_str.replace("\\", "/");
				match &uses_to_remove {
					None => changes.push(format!("Restored all uses to {path_str}.")),
					Some(gained_uses) => {
						changes.push(format!("Restored {gained_uses} uses to {path_str}."))
					}
				}
				for data_path in &entry.data_paths {
					let new_value = match &uses_to_remove {
						None => None,
						Some(gained_uses) => {
							let prev_value = persistent.get_first_selection_at::<u32>(data_path);
							let prev_value = prev_value.map(Result::ok).flatten().unwrap_or(0);
							let new_value = prev_value.saturating_add(*gained_uses);
							Some(new_value.to_string())
						}
					};
					persistent.set_selected(data_path, new_value);
				}
			}

			// TODO: These changes can be recorded in a commit when its time to save data.
			log::debug!("{changes:?}");

			close_modal.emit(());
			MutatorImpact::None
		}
	});

	let can_take_rest = hit_dice_to_consume.has_valid_input();

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{value}{" Rest"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div class="text-block">{value.description()}</div>
			{(*value == Rest::Short).then(|| html!(
				<HitDiceSection value={hit_dice_to_consume.clone()} />
			)).unwrap_or_default()}
			<ProjectedRestorations value={*value} />
			<div class="d-flex justify-content-center">
				<button class="btn btn-success" disabled={!can_take_rest} onclick={commit_rest}>
					{"Take "}{value}{" Rest"}
				</button>
			</div>
		</div>
	</>}
}

#[derive(Clone, PartialEq, Default)]
struct HitDiceToConsume {
	by_class: HashMap<String, (u32, PathBuf)>,
	total_rolls: RollSet,
	rolled_hp: u32,
}
impl HitDiceToConsume {
	fn add(&mut self, class_name: &str, die: Die, delta: i32, data_path: &PathBuf) {
		match self.by_class.get_mut(class_name) {
			None if delta > 0 => {
				self.by_class
					.insert(class_name.to_owned(), (delta as u32, data_path.clone()));
			}
			Some((die_count, _data_path)) if delta > 0 => {
				*die_count = die_count.saturating_add(delta as u32);
			}
			Some((die_count, _data_path)) if delta < 0 => {
				*die_count = die_count.saturating_sub(-delta as u32);
			}
			_ => {}
		}
		let delta_roll = Roll::from((delta.abs() as u32, die));
		match delta > 0 {
			true => self.total_rolls.push(delta_roll),
			false => self.total_rolls.remove(delta_roll),
		}
	}

	fn is_empty(&self) -> bool {
		self.total_rolls.is_empty()
	}

	fn has_valid_input(&self) -> bool {
		self.total_num_rolls() <= 0 || self.rolled_hp > 0
	}

	fn as_equation_str(&self) -> String {
		self.total_rolls.as_nonzero_string().unwrap_or_default()
	}

	fn total_num_rolls(&self) -> u32 {
		self.total_rolls.min()
	}

	fn hp_to_gain(&self, constitution_mod: i32) -> u32 {
		let roll_count = self.total_num_rolls() as i32;
		((self.rolled_hp as i32) + roll_count * constitution_mod).max(0) as u32
	}

	fn consumed_uses(&self) -> impl Iterator<Item = &(u32, PathBuf)> + '_ {
		self.by_class.values()
	}
}

#[function_component]
fn HitDiceSection(props: &GeneralProp<UseStateHandle<HitDiceToConsume>>) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let bonus_per_use = state.ability_modifier(Ability::Constitution, None);

	let hit_dice_to_consume = props.value.clone();
	let set_rolled_hp = Callback::from({
		let hit_dice_to_consume = hit_dice_to_consume.clone();
		move |evt: web_sys::Event| {
			let Some(rolled_hp) = evt.input_value_t::<u32>() else { return; };
			let mut value = (*hit_dice_to_consume).clone();
			value.rolled_hp = rolled_hp;
			hit_dice_to_consume.set(value);
		}
	});

	let on_dice_to_consume_changed = Callback::from({
		let hit_dice_to_consume = hit_dice_to_consume.clone();
		move |(class_name, die, delta, data_path): (AttrValue, Die, i32, Arc<PathBuf>)| {
			let mut dice_map = (*hit_dice_to_consume).clone();
			dice_map.add(class_name.as_str(), die, delta, &*data_path);
			hit_dice_to_consume.set(dice_map);
		}
	});

	let mut hit_die_usage_inputs = EnumMap::<Die, Vec<Html>>::default();
	for class in &state.persistent().classes {
		hit_die_usage_inputs[class.hit_die].push(html! {
			<div class="uses d-flex">
				<span class="d-inline-block me-4" style="width: 100px; font-weight: 600px;">
					{&class.name}{format!(" ({})", class.hit_die)}
				</span>
				<HitDiceUsageInput
					max_uses={class.current_level as u32}
					data_path={class.hit_die_selector.get_data_path()}
					on_change={on_dice_to_consume_changed.reform({
						let class_name: AttrValue = class.name.clone().into();
						let die = class.hit_die;
						let data_path = Arc::new(class.hit_die_selector.get_data_path().unwrap_or_default());
						move |delta: i32| (class_name.clone(), die, delta, data_path.clone())
					})}
				/>
			</div>
		});
	}

	let mut class_sections = Vec::new();
	for (_, mut inputs) in hit_die_usage_inputs.into_iter().rev() {
		class_sections.append(&mut inputs);
	}

	let rolled_hp_section = match hit_dice_to_consume.is_empty() {
		true => Html::default(),
		false => html!(<div class="mt-2">
			<h5>{"Rolled Hit Points"}</h5>
			<span class="me-2">
				{"Roll "}
				<i>{hit_dice_to_consume.as_equation_str()}</i>
				{" and type the resulting sum."}
			</span>
			<div class="d-flex justify-content-center">
				<input
					type="number" class="form-control text-center ms-3"
					style="font-size: 20px; padding: 0; height: 30px; width: 80px;"
					min="0"
					value={format!("{}", hit_dice_to_consume.rolled_hp)}
					onkeydown={validate_uint_only()}
					onchange={set_rolled_hp}
				/>
			</div>
			<span class="text-block">
				{format!(
					"You will gain {} hit points.\n({} rolled HP + {} * {:+} constitution modifier)",
					hit_dice_to_consume.hp_to_gain(bonus_per_use),
					hit_dice_to_consume.rolled_hp,
					hit_dice_to_consume.total_num_rolls(), bonus_per_use
				)}
			</span>
		</div>),
	};

	html!(<div class="mt-3">
		<h4>{"Hit Dice"}</h4>
		<span>
			{"Hit Dice are restored on a Long Rest. \
			Using a hit die restores the rolled amount of hit points \
			+ your constitution modifier per hit die rolled."}
		</span>
		<div class="mt-2">{class_sections}</div>
		{rolled_hp_section}
	</div>)
}

#[derive(Clone, PartialEq, Properties)]
struct HitDiceUsageInputProps {
	max_uses: u32,
	data_path: Option<PathBuf>,
	on_change: Callback<i32>,
}
#[function_component]
fn HitDiceUsageInput(
	HitDiceUsageInputProps {
		max_uses,
		data_path,
		on_change,
	}: &HitDiceUsageInputProps,
) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let uses_to_consume = use_state_eq(|| 0u32);
	let consumed_uses = data_path
		.as_ref()
		.map(|path| state.get_first_selection_at::<u32>(path));
	let consumed_uses = consumed_uses
		.flatten()
		.map(Result::ok)
		.flatten()
		.unwrap_or(0);
	let set_consumed_uses_delta = Callback::from({
		let uses_to_consume = uses_to_consume.clone();
		let on_change = on_change.clone();
		move |delta: i32| {
			let value = ((*uses_to_consume as i32) + delta).max(0) as u32;
			uses_to_consume.set(value);
			on_change.emit(delta);
		}
	});
	html! {
		<div class="uses">
			{match max_uses {
				0 => Html::default(),
				// we use checkboxes for anything <= 5 max uses
				1..=5 => {
					let toggle_use = Callback::from({
						let set_consumed_uses_delta = set_consumed_uses_delta.clone();
						move |evt: web_sys::Event| {
							let Some(consume_use) = evt.input_checked() else { return; };
							set_consumed_uses_delta.emit(consume_use.then_some(1).unwrap_or(-1));
						}
					});

					html! {<>
						{(0..*max_uses)
							.map(|idx| {
								html! {
									<input
										class={"form-check-input slot"} type={"checkbox"}
										checked={idx < consumed_uses + *uses_to_consume}
										disabled={idx < consumed_uses}
										onclick={stop_propagation()}
										onchange={toggle_use.clone()}
									/>
								}
							})
							.collect::<Vec<_>>()}
					</>}
				}
				// otherwise we use a numerical counter form
				_ => {
					let onclick_sub = set_consumed_uses_delta.reform(|_| -1);
					let onclick_add = set_consumed_uses_delta.reform(|_| 1);
					html! {
						<span class="deltaform d-flex align-items-center" onclick={stop_propagation()}>
							<button type="button" class="btn btn-theme sub" onclick={onclick_sub} disabled={*uses_to_consume == 0} />
							<span class="amount">{format!(
								"{} / {} ({} already spent)",
								*uses_to_consume,
								*max_uses - consumed_uses,
								consumed_uses,
							)}</span>
							<button type="button" class="btn btn-theme add" onclick={onclick_add} disabled={consumed_uses + *uses_to_consume >= *max_uses} />
						</span>
					}
				}
			}}
		</div>
	}
}

#[function_component]
fn ProjectedRestorations(GeneralProp { value }: &GeneralProp<Rest>) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let mut sections = Vec::new();
	match *value {
		Rest::Long => {
			sections.push(html!(<li style="color: var(--bs-warning);">{"WARNING: Your death saving throws will be reset."}</li>));
			sections.push(html!(<li>{"Regain all lost hit points."}</li>));
			sections.push(html!(<li>{"Temporary Hit Points will reset to 0."}</li>));

			let mut budget = 0;
			let mut max_hit_dice = EnumMap::<Die, usize>::default();
			for class in &state.persistent().classes {
				budget += class.current_level;
				max_hit_dice[class.hit_die] += class.current_level;
			}
			let mut hit_die_halves = Vec::with_capacity(max_hit_dice.len());
			for (die, total) in max_hit_dice.into_iter().rev() {
				if budget <= 0 {
					continue;
				}
				let total = total / 2;
				let total = match total.cmp(&budget) {
					std::cmp::Ordering::Less => {
						budget = budget.saturating_sub(total);
						total
					}
					_ => std::mem::take(&mut budget),
				};
				if total > 0 {
					hit_die_halves.push(Roll::from((total as u32, die)).to_string());
				}
			}
			if let Some(hit_dice) = crate::utility::list_as_english(hit_die_halves, "and") {
				sections.push(html!(<li>{format!("Regain up to {hit_dice} hit dice (half your total hit dice, minimuim of 1).")}</li>));
			}
		}
		Rest::Short => {}
	}
	for entry in state.rest_resets().get(*value) {
		let amt = match &entry.restore_amount {
			None => "all".to_owned(),
			Some(roll) => roll.to_string(),
		};
		let path_str = crate::data::as_feature_path_text(&entry.source).unwrap_or_default();
		let description = format!("Restore {amt} uses of {path_str}.");
		sections.push(html!(<li>{description}</li>));
	}
	html! {
		<div class="mt-3">
			<h4>{"Affected Features"}</h4>
			{match sections.is_empty() {
				false => html!(<ul>{sections}</ul>),
				true => html!("No other changes will be made."),
			}}
		</div>
	}
}
