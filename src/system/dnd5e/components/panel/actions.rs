use crate::{
	components::{
		database::{use_query_typed, QueryStatus},
		*,
	},
	page::characters::sheet::{
		joined::editor::{description, mutator_list},
		CharacterHandle, MutatorImpact,
	},
	system::{
		core::SourceId,
		dnd5e::{
			components::UsesCounter,
			data::{
				action::{ActivationKind, AttackCheckKind, AttackKindValue},
				character::{ActionBudgetKind, Persistent},
				AreaOfEffect, Condition, DamageRoll, Feature, IndirectCondition,
			},
		},
	},
};
use enum_map::{Enum, EnumMap};
use enumset::{EnumSet, EnumSetType};
use itertools::{Itertools, Position};
use multimap::MultiMap;
use std::{collections::HashMap, path::PathBuf};
use yew::prelude::*;

#[derive(EnumSetType, Enum)]
pub enum ActionTag {
	Attack,
	Action,
	BonusAction,
	Reaction,
	Other,
	LimitedUse,
	Passive,
}
impl From<ActivationKind> for ActionTag {
	fn from(value: ActivationKind) -> Self {
		match value {
			ActivationKind::Action => Self::Action,
			ActivationKind::Bonus => Self::BonusAction,
			ActivationKind::Reaction => Self::Reaction,
			ActivationKind::Special => Self::Other,
			ActivationKind::Minute(_) => Self::Other,
			ActivationKind::Hour(_) => Self::Other,
		}
	}
}
impl ActionTag {
	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Attack => "Attack",
			Self::Action => "Action",
			Self::BonusAction => "Bonus Action",
			Self::Reaction => "Reaction",
			Self::Other => "Other",
			Self::LimitedUse => "Limited Use",
			Self::Passive => "Passive",
		}
	}
}

#[function_component]
pub fn Actions() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let selected_tags = use_state(|| EnumSet::<ActionTag>::all());

	let context_menu = use_context::<context_menu::Control>().unwrap();
	let open_feature_details = Callback::from({
		let context_menu = context_menu.clone();
		let state = state.clone();
		move |feature_path: PathBuf| {
			let Some(feature) = state.features().path_map.get_first(&feature_path) else { return; };
			context_menu.dispatch(context_menu::Action::open_root(
				feature.name.clone(),
				html!(<Modal path={feature_path} />),
			));
		}
	});

	let make_tag_html = {
		let selected_tags = selected_tags.clone();
		move |html: Html, tag_set: EnumSet<ActionTag>| {
			let active = *selected_tags == tag_set;
			let on_click = {
				let selected_tags = selected_tags.clone();
				Callback::from(move |_| selected_tags.set(tag_set))
			};
			html! { <Tag {active} {on_click}>{html}</Tag> }
		}
	};
	let mut tag_htmls = vec![make_tag_html(html! {{"All"}}, EnumSet::all())];
	for tag in EnumSet::<ActionTag>::all() {
		tag_htmls.push(make_tag_html(
			html! {{tag.display_name()}},
			EnumSet::from(tag),
		));
	}

	let budget = {
		// TODO: Modal for action budget
		let mut budget_items = Vec::new();
		{
			let (amount, _) = state.features().action_budget.get(ActionBudgetKind::Action);
			budget_items.push((ActionBudgetKind::Action, amount));
		}
		if selected_tags.contains(ActionTag::Attack) {
			let (amount, _) = state.features().action_budget.get(ActionBudgetKind::Attack);
			budget_items.push((ActionBudgetKind::Attack, amount));
		}
		{
			let (amount, _) = state.features().action_budget.get(ActionBudgetKind::Bonus);
			budget_items.push((ActionBudgetKind::Bonus, amount));
		}
		{
			let (amount, _) = state
				.features()
				.action_budget
				.get(ActionBudgetKind::Reaction);
			budget_items.push((ActionBudgetKind::Reaction, amount));
		}
		html! {
			<span class="action-budget">
				{"Action Budget: "}
				{budget_items.into_iter().map(|(kind, amount)| match kind {
					ActionBudgetKind::Action => format!("{amount} Actions"),
					ActionBudgetKind::Attack => format!("{amount} Attacks per Action"),
					ActionBudgetKind::Bonus => format!("{amount} Bonus Actions"),
					ActionBudgetKind::Reaction => format!("{amount} Reactions"),
				}).collect::<Vec<_>>().join(", ")}
			</span>
		}
	};

	let mut panes = Vec::new();
	if selected_tags.contains(ActionTag::Attack) {
		let features = {
			let mut features = state
				.features()
				.iter_all()
				.filter(|(_, feature)| match feature.action.as_ref() {
					Some(action) => action.attack.is_some(),
					None => false,
				})
				.collect::<Vec<_>>();
			features.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
			features
		};

		panes.push(html! {
			<table class="table table-compact m-0 mb-3">
				<thead>
					<tr class="text-center" style="font-size: 0.7rem;">
						<th scope="col">{"Attack"}</th>
						<th scope="col">{"Range"}</th>
						<th scope="col">{"Hit / DC"}</th>
						<th scope="col">{"Damage"}</th>
						<th scope="col">{"Notes"}</th>
					</tr>
				</thead>
				<tbody>
					{features.into_iter().filter_map(|(feature_path, feature)| {
						let Some(action) = &feature.action else {
							return None;
						};
						let Some(attack) = &action.attack else {
							return None;
						};
						let onclick = open_feature_details.reform(move |_| feature_path.clone());

						let (_check_ability, atk_bonus, dmg_bonus) = attack.evaluate_bonuses(&*state);
						Some(html! {
							<tr class="align-middle" {onclick}>
								<td>{feature.name.clone()}</td>
								<td>{match &attack.kind {
									None => html!(),
									Some(AttackKindValue::Melee { reach }) => html! {<>{reach}{"ft."}</>},
									Some(AttackKindValue::Ranged { short_dist, long_dist, .. }) => html! {<>{short_dist}{" / "}{long_dist}</>},
								}}</td>
								<td class="text-center">{{
									match attack.check {
										AttackCheckKind::AttackRoll {..} => html!{<>
											{match atk_bonus >= 0 { true => "+", false => "-" }}
											{atk_bonus.abs()}
										</>},
										AttackCheckKind::SavingThrow { save_ability, ..} => html!{<>
											{save_ability.abbreviated_name()}
											<br />
											{atk_bonus}
										</>},
									}
								}}</td>
								<td class="text-center">{{
									//let additional_damage = state.attack_bonuses().get_weapon_damage(action);
									match &attack.damage {
										Some(DamageRoll { roll, base_bonus, damage_type: _ }) => {
											// TODO: Showing bonuses when a bonus is a roll with an optional damage type
											//let additional_bonus: i32 = additional_damage.iter().map(|(v, _)| *v).sum();
											let bonus = base_bonus + dmg_bonus;// + additional_bonus;
											let roll_str = match &roll {
												None => None,
												Some(roll_value) => Some(roll_value.evaluate(&state).to_string()),
											};
											match (roll_str, bonus) {
												(None, bonus) => html! {{bonus.max(0)}},
												(Some(roll), 0) => html! {{roll}},
												(Some(roll), 1..=i32::MAX) => html! {<>{roll}{" + "}{bonus}</>},
												(Some(roll), i32::MIN..=-1) => html! {<>{roll}{" - "}{bonus.abs()}</>},
											}
										}
										None => html! {},
									}
								}}</td>
								<td style="width: 200px;"></td>
							</tr>
						})
					}).collect::<Vec<_>>()}
				</tbody>
			</table>
		});
	}

	let (root_features, collapsed_features) = {
		let mut root_features = FeatureDisplayGroup::default();
		let mut collapsed_features = FeatureDisplayGroup::default();
		let mut features_by_parent = MultiMap::new();
		for (feature_path, feature) in state.features().iter_all() {
			let mut passes_any = false;
			if let Some(action) = &feature.action {
				if selected_tags.contains(ActionTag::Action) {
					passes_any = passes_any || action.activation_kind == ActivationKind::Action;
				}
				if selected_tags.contains(ActionTag::BonusAction) {
					passes_any = passes_any || action.activation_kind == ActivationKind::Bonus;
				}
				if selected_tags.contains(ActionTag::Reaction) {
					passes_any = passes_any || action.activation_kind == ActivationKind::Reaction;
				}
				if selected_tags.contains(ActionTag::Other) {
					let is_regular_action = matches!(
						action.activation_kind,
						ActivationKind::Action | ActivationKind::Bonus | ActivationKind::Reaction
					);
					passes_any = passes_any || !is_regular_action;
				}
				if selected_tags.contains(ActionTag::LimitedUse) {
					passes_any = passes_any || action.limited_uses.is_some();
				}
				// Has an action, we can include this feature depending on the action types that we are displaying
				if let Some(attack) = &action.attack {
					if attack.weapon_kind.is_some() {
						// weapon attacks are hidden from general feature listings (they only show as attacks)
						passes_any = false;
					}
				}
			} else {
				// No action, we can include this feature if we are displaying passive features.
				passes_any = passes_any || selected_tags.contains(ActionTag::Passive);
			}

			let entry = FeatureEntry {
				feature_path,
				// cloning the feature is required to pass the feature to a yew component later
				feature: feature.clone(),
				children: FeatureDisplayGroup::default(),
				is_relevant: passes_any,
			};
			// If this feature has a display parent, cache it off until all of the
			// top-level features have been gathered (a feature with a display parent
			// may be encountered before the parent itself).
			if let Some(display_parent_path) = &feature.parent {
				if entry.is_relevant {
					features_by_parent.insert(display_parent_path.clone(), entry);
				}
			} else {
				if feature.collapsed {
					collapsed_features.insert(entry);
				} else {
					root_features.insert(entry);
				}
			}
		}
		for (parent_path, child_features) in features_by_parent.into_iter() {
			if let Some(prev_collapsed) = collapsed_features.remove(&parent_path) {
				root_features.insert(prev_collapsed);
			}
			let Some(parent_group) = root_features.by_path.get_mut(&parent_path) else {
				log::warn!("Found features with the parent path \"{}\", but no such feature exists", parent_path.display());
				continue;
			};
			for entry in child_features {
				parent_group.children.insert(entry);
			}
		}
		(root_features, collapsed_features)
	};
	panes.push(html! {<div>
		{(!collapsed_features.is_empty()).then(move || {
			let mut entries_by_tag = EnumMap::<ActionTag, Vec<Html>>::default();
			for entry in collapsed_features.into_iter() {
				let tag = match &entry.feature.action {
					Some(action) => ActionTag::from(action.activation_kind),
					None => ActionTag::Passive,
				};
				entries_by_tag[tag].push(html! { <CollapsedFeature {entry} /> });
			}
			html! {
				<div class="mb-2 border-bottom-theme-muted">
					<strong>{"Other Features"}</strong>
					<div class="pb-1 ms-3" style="font-size: 12px;">
						{entries_by_tag.into_iter().filter_map(|(tag, nodes)| {
							if nodes.is_empty() {
								return None;
							}
							Some(html! {
								<div>
									<strong>{tag.display_name()}{": "}</strong>
									<span>
										{nodes.into_iter().with_position().map(|position| {
											{match position {
												Position::Only(node) | Position::Last(node) => node,
												Position::First(node) | Position::Middle(node) => html! {
													<span>{node}{", "}</span>
												},
											}}
										}).collect::<Vec<_>>()}
									</span>
								</div>
							})
						}).collect::<Vec<_>>()}
					</div>
				</div>
			}
		}).unwrap_or_default()}
		{root_features.into_iter().filter_map(|entry| {
			// some features are not relevant to this view, but have children which are.
			// If its not relevant AND doesn't have children, then we shouldnt display it.
			if !entry.is_relevant && entry.children.is_empty() {
				return None;
			}
			Some(html! { <ActionOverview {entry} /> })
		}).collect::<Vec<_>>()}
	</div>});

	html! {
		<div class="panel actions">
			<Tags>{tag_htmls}</Tags>
			{budget}
			<div class="pane">
				{panes}
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Default)]
struct FeatureDisplayGroup {
	// the order of entries, identified by the path of a feature,
	// sorted alphabetically by feature name
	order: Vec<PathBuf>,
	// features keyed by their unique paths
	by_path: HashMap<PathBuf, FeatureEntry>,
}
#[derive(Clone, PartialEq)]
struct FeatureEntry {
	feature_path: PathBuf,
	feature: Feature,
	children: FeatureDisplayGroup,
	is_relevant: bool,
}
impl FeatureDisplayGroup {
	fn insert(&mut self, entry: FeatureEntry) {
		// Insertion sort the feature_path into the alphabetical order
		let insert_idx = self.order.binary_search_by(|path_to_existing| {
			let existing_entry = self.by_path.get(path_to_existing).unwrap();
			let cmp_name = existing_entry.feature.name.cmp(&entry.feature.name);
			let cmp_path = existing_entry.feature_path.cmp(&entry.feature_path);
			cmp_name.then(cmp_path)
		});
		let insert_idx = match insert_idx {
			Ok(idx) => idx,  // was found, but thats fineTM, we'll just ignore it
			Err(idx) => idx, // not found, we can insert here
		};
		self.order.insert(insert_idx, entry.feature_path.clone());
		self.by_path.insert(entry.feature_path.clone(), entry);
	}

	fn remove(&mut self, path: &PathBuf) -> Option<FeatureEntry> {
		let value = self.by_path.remove(path);
		if value.is_some() {
			self.order.retain(|p| p != path);
		}
		value
	}

	fn is_empty(&self) -> bool {
		self.order.is_empty()
	}

	fn iter(&self) -> impl Iterator<Item = &FeatureEntry> {
		self.order
			.iter()
			.filter_map(move |path| self.by_path.get(path))
	}

	fn into_iter(mut self) -> impl Iterator<Item = FeatureEntry> {
		self.order
			.into_iter()
			.filter_map(move |path| self.by_path.remove(&path))
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ActionProps {
	pub entry: FeatureEntry,
}

#[function_component]
fn CollapsedFeature(ActionProps { entry }: &ActionProps) -> Html {
	let onclick = context_menu::use_control_action({
		let feature_path: PathBuf = entry.feature_path.clone();
		let feature_name = AttrValue::from(entry.feature.name.clone());
		move |_| {
			context_menu::Action::open_root(
				feature_name.clone(),
				html!(<Modal path={feature_path.clone()} />),
			)
		}
	});
	html! {
		<span {onclick}>{entry.feature.name.clone()}</span>
	}
}

#[function_component]
fn ActionOverview(ActionProps { entry }: &ActionProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();

	let onclick = context_menu::use_control_action({
		let feature_path: PathBuf = entry.feature_path.clone();
		let feature_name = AttrValue::from(entry.feature.name.clone());
		move |_| {
			context_menu::Action::open_root(
				feature_name.clone(),
				html!(<Modal path={feature_path.clone()} />),
			)
		}
	});

	let fetch_indirect_conditions = use_query_typed::<Condition>();
	let indirect_condition_ids = use_state_eq(|| Vec::new());
	use_effect_with_deps(
		{
			let fetch_indirect_conditions = fetch_indirect_conditions.clone();
			move |ids: &UseStateHandle<Vec<SourceId>>| {
				fetch_indirect_conditions.run((**ids).clone());
			}
		},
		indirect_condition_ids.clone(),
	);

	let mut conditions_content = html!();
	if let Some(action) = &entry.feature.action {
		if !action.conditions_to_apply.is_empty() {
			indirect_condition_ids.set({
				let iter_conditions = action.conditions_to_apply.iter();
				let iter_conditions = iter_conditions.filter_map(|indirect| match indirect {
					IndirectCondition::Id(id) => Some(id.clone()),
					IndirectCondition::Custom(_custom) => None,
				});
				iter_conditions.collect::<Vec<_>>()
			});

			conditions_content = match fetch_indirect_conditions.status() {
				QueryStatus::Pending => html!(<Spinner />),
				status => {
					let fetched_conditions = match status {
						QueryStatus::Success((_, items)) => Some(items),
						_ => None,
					};
					let iter = action.conditions_to_apply.iter();
					let iter = iter.filter_map(|indirect| match indirect {
						IndirectCondition::Custom(condition) => Some(condition.clone()),
						IndirectCondition::Id(id) => fetched_conditions
							.map(|list| list.get(id))
							.flatten()
							.cloned(),
					});
					let conditions = iter.collect::<Vec<_>>();

					let onclick = Callback::from({
						let state = state.clone();
						let conditions_to_apply = conditions.clone();
						move |evt: MouseEvent| {
							evt.stop_propagation();
							let conditions_to_apply = conditions_to_apply.clone();
							state.dispatch(Box::new(move |persistent: &mut Persistent| {
								// TODO: Applying a condition should include the path to the feature which caused it (if it was not manually added)
								for condition in &*conditions_to_apply {
									persistent.conditions.insert(condition.clone());
								}
								MutatorImpact::Recompile
							}));
						}
					});

					let count = conditions.len();
					html! {
						<div class="conditions d-flex align-items-baseline">
							<strong class="title">{"Applies Conditions:"}</strong>
							<div class="mx-2">
								{conditions.iter().enumerate().map(|(idx, condition)| html! {
									<span>
										{&condition.name}
										{match /*is_last*/ idx == count - 1 {
											false => ", ",
											true => "",
										}}
									</span>
								}).collect::<Vec<_>>()}
							</div>
							<button
								type="button" class="btn btn-primary btn-xs ms-auto"
								{onclick}
							>
								{"Apply"}
							</button>
						</div>
					}
				}
			};
		}
	}

	let action_block = match &entry.feature.action {
		None => html! {},
		Some(action) => html! {
			<div class="addendum mx-2 mb-1">
				{conditions_content}
				{action.limited_uses.as_ref().map(|limited_uses| {
					UsesCounter { state: state.clone(), limited_uses }.to_html()
				}).unwrap_or_default()}
			</div>
		},
	};
	let same_name_as_display_parent = match &entry.feature.parent {
		None => false,
		Some(parent) => match parent.file_name() {
			None => false,
			Some(parent_name) => parent_name.to_str() == Some(&entry.feature.name),
		},
	};
	let desc = entry.feature.description.clone().evaluate(&state);
	html! {
		<div class="feature short pb-1" {onclick}>
			{(!same_name_as_display_parent).then(|| html! {
				<strong class="title">{entry.feature.name.clone()}</strong>
			}).unwrap_or_default()}
			<span class="subtitle">
				<span style="margin-right: 5px;">
					{match &entry.feature.action {
						Some(action) => html! { {action.activation_kind} },
						None => html! { "Passive" },
					}}
				</span>
				{match entry.feature_path.parent() {
					Some(parent_path) if parent_path.components().count() > 0 => html! {
						<span>{"("}{crate::data::as_feature_path_text(parent_path)}{")"}</span>
					},
					_ => html! {},
				}}
			</span>
			{description(&desc, true, false)}
			{action_block}
			{match entry.children.is_empty() {
				true => html! {},
				false => html! {
					<div class="children ms-3 mb-1" onclick={stop_propagation()}>
						{entry.children.iter().cloned().map(|entry| {
							html! { <ActionOverview {entry} /> }
						}).collect::<Vec<_>>()}
					</div>
				}
			}}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ModalProps {
	// The path to the feature in `state.features().path_map`.
	pub path: PathBuf,
}
#[function_component]
fn Modal(ModalProps { path }: &ModalProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let fetch_indirect_conditions = use_query_typed::<Condition>();
	let indirect_condition_ids = use_state_eq(|| Vec::new());
	use_effect_with_deps(
		{
			let fetch_indirect_conditions = fetch_indirect_conditions.clone();
			move |ids: &UseStateHandle<Vec<SourceId>>| {
				fetch_indirect_conditions.run((**ids).clone());
			}
		},
		indirect_condition_ids.clone(),
	);

	let Some(feature) = state.features().path_map.get_first(&path) else {
		return html! {<>
			{"Missing Feature: "}
			{crate::data::as_feature_path_text(&path)}
		</>};
	};

	let mut sections = Vec::new();

	if let Some(parent) = path.parent() {
		if parent.components().count() > 0 {
			sections.push(html! {
				<div class="property">
					<strong>{"Source:"}</strong>
					<span>{crate::data::as_feature_path_text(parent)}</span>
				</div>
			});
		}
	}

	if let Some(action) = &feature.action {
		let mut action_sections = Vec::new();

		action_sections.push(html! {
			<div class="property">
				<strong>{"Action Type:"}</strong>
				<span>{action.activation_kind}</span>
			</div>
		});

		if let Some(attack) = &action.attack {
			let mut attack_sections = Vec::new();

			match &attack.kind {
				None => {}
				Some(AttackKindValue::Melee { reach }) => {
					attack_sections.push(html! {
						<div class="property">
							<strong>{"Kind:"}</strong>
							<span>{"Melee"}</span>
						</div>
					});
					attack_sections.push(html! {
						<div class="property">
							<strong>{"Range:"}</strong>
							<span>{format!("{reach} ft.")}</span>
						</div>
					});
				}
				// TODO: find a way to communicate attack range better:
				// - normal if the target is at or closer than `short`
				// - made a disadvantage when the target is father than `short`, but closer than `long`
				// - impossible beyond the `long` range
				Some(AttackKindValue::Ranged {
					short_dist,
					long_dist,
				}) => {
					attack_sections.push(html! {
						<div class="property">
							<strong>{"Kind:"}</strong>
							<span>{"Ranged"}</span>
						</div>
					});
					attack_sections.push(html! {
						<div class="property">
							<strong>{"Range:"}</strong>
							<span>
								{format!("{short_dist} ft / {long_dist} ft")}
							</span>
						</div>
					});
				}
			}

			let (check_ability, atk_bonus, dmg_bonus) = attack.evaluate_bonuses(&*state);
			let check_ability_mod_str = check_ability
				.map(|ability| format!("{} modifier", ability.long_name()))
				.unwrap_or_default();
			match &attack.check {
				AttackCheckKind::AttackRoll {
					ability: _,
					proficient,
				} => {
					let use_prof = proficient.evaluate(&*state);
					attack_sections.push(html! {
						<div class="property">
							<strong>{"To Hit:"}</strong>
							<span>
								{match atk_bonus >= 0 { true => "+", false => "-" }}
								{atk_bonus.abs()}
							</span>
							<span style="color: var(--bs-gray-600);">
								{" ("}
								{&check_ability_mod_str}
								{use_prof.then(|| html! { {" + proficiency bonus"} }).unwrap_or_default()}
								{")"}
							</span>
						</div>
					});
				}
				AttackCheckKind::SavingThrow {
					base,
					dc_ability,
					proficient,
					save_ability,
				} => {
					attack_sections.push(html! {
						<div class="property">
							<strong>{"Saving Throw:"}</strong>
							<span>
								{format!("{} - DC {atk_bonus}", save_ability.long_name())}
							</span>
							<span class="ms-1" style="color: var(--bs-gray-600);">
								{"("}
								{(*base > 0).then(|| html! { {format!("{base}")} }).unwrap_or_default()}
								{dc_ability.as_ref().map(|ability| html! {
									{format!(" + {} modifier", ability.long_name())}
								}).unwrap_or_default()}
								{proficient.then(|| html! { {" + proficiency bonus"} }).unwrap_or_default()}
								{")"}
							</span>
						</div>
					});
				}
			}

			if let Some(area_of_effect) = &attack.area_of_effect {
				attack_sections.push(html! {
					<div class="property">
						<strong>{"Area of Effect:"}</strong>
						<span>
							{match area_of_effect {
								AreaOfEffect::Cone { length } => format!("Cone ({length} ft)"),
								AreaOfEffect::Cube { size } => format!("Cube ({size} ft)"),
								AreaOfEffect::Cylinder { radius, height } => format!("Cylinder ({radius} ft. radius, {height} ft. height)"),
								AreaOfEffect::Line { width, length } => format!("Line ({width} ft. width, {length} ft. length)"),
								AreaOfEffect::Sphere { radius } => format!("Sphere ({radius} ft)"),
							}}
						</span>
					</div>
				});
			}

			//let additional_damage = state.attack_bonuses().get_weapon_damage(action);
			if let Some(DamageRoll {
				roll,
				base_bonus,
				damage_type,
			}) = &attack.damage
			{
				// TODO: Show damage roll bonuses inline, when the bonuses themselves can be rolls
				//let additional_bonus: i32 = additional_damage.iter().map(|(v, _damage_type, _source)| *v).sum();
				let bonus = base_bonus + dmg_bonus; // + additional_bonus;
				let roll_str = match &roll {
					None => None,
					Some(roll_value) => Some(roll_value.evaluate(&state).to_string()),
				};
				let concat_roll_bonus =
					|roll_str: &Option<String>, bonus: i32| match (&roll_str, bonus) {
						(None, bonus) => html! {{bonus.max(0)}},
						(Some(roll), 0) => html! {{roll}},
						(Some(roll), 1..=i32::MAX) => html! {<>{roll}{" + "}{bonus}</>},
						(Some(roll), i32::MIN..=-1) => html! {<>{roll}{" - "}{bonus.abs()}</>},
					};
				let additional_damage_html = html!() /*additional_damage.iter().map(|(value, _damage_type, source)| html! {
					<span>
						{match *value >= 0 { true => "+", false => "-" }}
						{value.abs()}
						{"("}{crate::data::as_feature_path_text(source).unwrap_or_default()}{")"}
					</span>
				}).collect::<Vec<_>>()*/;
				let suffix_info = (bonus > 0 && bonus != *base_bonus).then(|| {
					html! {
						<span style="color: var(--bs-gray-600);">
							{" ("}
							{concat_roll_bonus(&roll_str, *base_bonus)}
							{" + "}
							{&check_ability_mod_str}
							{additional_damage_html}
							{")"}
						</span>
					}
				});
				attack_sections.push(html! {
					<div class="property">
						<strong>{"Damage:"}</strong>
						<span>
							{concat_roll_bonus(&roll_str, bonus)}{format!(" {}", damage_type.display_name())}
						</span>
						{suffix_info.unwrap_or_default()}
					</div>
				});
			}

			if let Some(weapon_kind) = &attack.weapon_kind {
				attack_sections.push(html! {
					<div class="property">
						<strong>{"Weapon Kind:"}</strong>
						<span>{weapon_kind.to_string()}</span>
					</div>
				});
			}

			action_sections.push(html! {
				<div class="ps-2 border-bottom-theme-muted">
					<strong>{"Attack"}</strong>
					<div>{attack_sections}</div>
				</div>
			});
		}

		if let Some(limited_uses) = &action.limited_uses {
			action_sections.push(
				UsesCounter {
					state: state.clone(),
					limited_uses,
				}
				.to_html(),
			);
		}

		if !action.conditions_to_apply.is_empty() {
			// Run the query with whatever conditions need to be fetched.
			// If the query is currently active, this will silently be ignored.
			// If the queried entries match the new query, this is a NO-OP and is silently ignored.
			indirect_condition_ids.set({
				let iter_conditions = action.conditions_to_apply.iter();
				let iter_conditions = iter_conditions.filter_map(|indirect| match indirect {
					IndirectCondition::Id(id) => Some(id.clone()),
					IndirectCondition::Custom(_custom) => None,
				});
				iter_conditions.collect::<Vec<_>>()
			});

			let condition_content = match fetch_indirect_conditions.status() {
				QueryStatus::Pending => html!(<Spinner />),
				status => {
					let fetched_conditions = match status {
						QueryStatus::Success((_ids, items)) => Some(items),
						_ => None,
					};
					let iter = action.conditions_to_apply.iter();
					let iter = iter.filter_map(|indirect| match indirect {
						IndirectCondition::Custom(custom) => Some(custom),
						IndirectCondition::Id(id) => fetched_conditions
							.map(|listings| listings.get(id))
							.flatten(),
					});
					let condition_nodes = iter.map(|condition| {
						html! {
							<div>
								<h6>{condition.name.clone()}</h6>
								{condition.description.clone()}
								<div>
									<strong>{"Effects:"}</strong>
									<div class="mx-2">
										{mutator_list(&condition.mutators, Some(&state))}
									</div>
								</div>
							</div>
						}
					});
					html!(<>{condition_nodes.collect::<Vec<_>>()}</>)
				}
			};

			action_sections.push(html! {
				<div class="conditions">
					<h5>{"Conditions Applied on Use"}</h5>
					{condition_content}
				</div>
			});
		}

		sections.push(html! {<>{action_sections}</>});
	}

	let desc = feature.description.clone().evaluate(&state);
	sections.push(description(&desc, false, false));

	if !feature.mutators.is_empty() {
		sections.push(html! {
			<div class="property">
				<strong>{"Alterations:"}</strong>
				<div>
					{mutator_list(&feature.mutators, Some(&state))}
				</div>
			</div>
		});
	}

	html! {<>
		<div class="details feature">
			{sections}
		</div>
	</>}
}
