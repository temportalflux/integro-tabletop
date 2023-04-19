use crate::{
	components::*,
	system::dnd5e::{
		components::{
			editor::{description, mutator_list},
			SharedCharacter, UsesCounter,
		},
		data::{
			action::{ActivationKind, AttackCheckKind, AttackKindValue},
			character::{ActionBudgetKind, ActionEffect, Persistent},
			DamageRoll, Feature,
		},
		DnD5e,
	},
};
use enumset::{EnumSet, EnumSetType};
use multimap::MultiMap;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use yew::prelude::*;

#[derive(EnumSetType)]
pub enum ActionTag {
	Attack,
	Action,
	BonusAction,
	Reaction,
	Other,
	LimitedUse,
	Passive,
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
	let state = use_context::<SharedCharacter>().unwrap();
	let selected_tags = use_state(|| EnumSet::<ActionTag>::all());

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
				.filter(|(_parent_path, feature)| match feature.action.as_ref() {
					Some(action) => action.attack.is_some(),
					None => false,
				})
				.map(|(_, feature)| feature)
				.collect::<Vec<_>>();
			features.sort_by(|a, b| a.name.cmp(&b.name));
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
					{features.into_iter().map(|feature| {
						let Some(action) = &feature.action else {
							return html! {};
						};
						let Some(attack) = &action.attack else {
							return html! {};
						};
						html! {
							<tr class="align-middle">
								<td>{feature.name.clone()}</td>
								<td>{match attack.kind {
									AttackKindValue::Melee { reach } => html! {<>{reach}{"ft."}</>},
									AttackKindValue::Ranged { short_dist, long_dist, .. } => html! {<>{short_dist}{" / "}{long_dist}</>},
								}}</td>
								<td class="text-center">{{
									let value = attack.check.evaluate(&*state);
									match attack.check {
										AttackCheckKind::AttackRoll {..} => html!{<>
											{match value >= 0 { true => "+", false => "-" }}
											{value.abs()}
										</>},
										AttackCheckKind::SavingThrow { save_ability, ..} => html!{<>
											{save_ability.abbreviated_name()}
											<br />
											{value}
										</>},
									}
								}}</td>
								<td class="text-center">{{
									let ability_bonus = match &attack.check {
										AttackCheckKind::AttackRoll { ability, .. } => state.ability_modifier(*ability, None),
										_ => 0,
									};
									match &attack.damage {
										// TODO: tooltip for where bonus come from
										Some(DamageRoll { roll, base_bonus, damage_type: _, additional_bonuses }) => {
											let additional_bonus: i32 = additional_bonuses.iter().map(|(v, _)| *v).sum();
											let bonus = base_bonus + ability_bonus + additional_bonus;
											let roll = roll.as_ref().map(|roll| html!{{roll.to_string()}});
											match (roll, bonus) {
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
						}
					}).collect::<Vec<_>>()}
				</tbody>
			</table>
		});
	}

	let features = {
		let mut root_features = FeatureDisplayGroup::default();
		let mut features_by_parent = MultiMap::new();
		for (feature_path, feature) in state.features().iter_all() {
			let mut passes_any = false;
			if let Some(action) = &feature.action {
				// Has an action, we can include this feature depending on the action types that we are displaying
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
			} else {
				// No action, we can include this feature if we are displaying passive features.
				passes_any = passes_any || selected_tags.contains(ActionTag::Passive);
			}
			if !passes_any {
				continue;
			}

			let entry = FeatureEntry {
				feature_path,
				// cloning the feature is required to pass the feature to a yew component later
				feature: feature.clone(),
				children: FeatureDisplayGroup::default(),
			};
			// If this feature has a display parent, cache it off until all of the
			// top-level features have been gathered (a feature with a display parent
			// may be encountered before the parent itself).
			if let Some(display_parent_path) = &feature.parent {
				features_by_parent.insert(display_parent_path.clone(), entry);
			}
			else {
				root_features.insert(entry);
			}
		}
		for (parent_path, child_features) in features_by_parent.into_iter() {
			let Some(parent_group) = root_features.by_path.get_mut(&parent_path) else {
				log::warn!("Found features with the parent path \"{}\", but no such feature exists", parent_path.display());
				continue;
			};
			for entry in child_features {
				parent_group.children.insert(entry);
			}
		}
		root_features
	};
	panes.push(html! {<div>
		{features.into_iter().map(|entry| {
			html! { <ActionOverview {entry} /> }
		}).collect::<Vec<_>>()}
	</div>});

	html! {<>
		<Tags>{tag_htmls}</Tags>
		{budget}
		<div style="overflow-y: scroll; height: 455px;">
			{panes}
		</div>
	</>}
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
}
impl FeatureDisplayGroup {
	fn insert(&mut self, entry: FeatureEntry) {
		// Insertion sort the feature_path into the alphabetical order
		let insert_idx = self.order.binary_search_by(|path_to_existing| {
			let existing_entry = self.by_path.get(path_to_existing).unwrap();
			existing_entry.feature.name.cmp(&entry.feature.name)
		});
		let insert_idx = match insert_idx {
			Ok(idx) => idx,  // was found, but thats fineTM, we'll just ignore it
			Err(idx) => idx, // not found, we can insert here
		};
		self.order.insert(insert_idx, entry.feature_path.clone());
		self.by_path.insert(entry.feature_path.clone(), entry);
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
fn ActionOverview(ActionProps { entry }: &ActionProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let onclick = modal_dispatcher.callback({
		let feature_path = entry.feature_path.clone();
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("feature"),
				content: html! {<Modal path={feature_path.clone()} />},
				..Default::default()
			})
		}
	});
	let action_block = match &entry.feature.action {
		None => html! {},
		Some(action) => html! {
			<div class="addendum mx-2 mb-1">
				{(!action.conditions_to_apply.is_empty()).then(|| {
					let conditions_to_apply = Arc::new(action.conditions_to_apply.iter().filter_map(|indirect| {
						indirect.resolve(&system).cloned()
					}).collect::<Vec<_>>());
					let name_section = {
						let count = conditions_to_apply.len();
						html! {
							<div class="mx-2">
								{conditions_to_apply.iter().enumerate().map(|(idx, condition)| html! {
									<span>
										{condition.name.clone()}
										{match /*is_last*/ idx == count - 1 {
											false => ", ",
											true => "",
										}}
									</span>
								}).collect::<Vec<_>>()}
							</div>
						}
					};
					let onclick = Callback::from({
						let state = state.clone();
						move |evt: MouseEvent| {
							evt.stop_propagation();
							let conditions_to_apply = conditions_to_apply.clone();
							state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
								// TODO: Applying a condition should include the path to the feature which caused it (if it was not manually added)
								for condition in &*conditions_to_apply {
									persistent.conditions.insert(condition.clone());
								}
								Some(ActionEffect::Recompile)
							}));
						}
					});
					html! {
						<div class="conditions d-flex align-items-baseline">
							<strong class="title">{"Applies Conditions:"}</strong>
							{name_section}
							<button
								type="button" class="btn btn-primary btn-xs ms-auto"
								{onclick}
							>
								{"Apply"}
							</button>
						</div>
					}
				}).unwrap_or_default()}
				{action.limited_uses.as_ref().map(|limited_uses| {
					UsesCounter { state: state.clone(), limited_uses }.to_html()
				}).unwrap_or_default()}
			</div>
		},
	};
	html! {
		<div class="feature short pb-1" {onclick}>
			<strong class="title">{entry.feature.name.clone()}</strong>
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
			{description(&entry.feature.description, true)}
			{action_block}
			{match entry.children.is_empty() {
				true => html! {},
				false => html! {
					<div class="children mx-2 mb-1" onclick={Callback::from(|evt: MouseEvent| evt.stop_propagation())}>
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
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let Some(feature) = state.features().path_map.get_first(&path) else {
		return html! {<>
			<div class="modal-header">
			<h1 class="modal-title fs-4">{"Missing Feature"}</h1>
				<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
			</div>
			<div class="modal-body">
				{crate::data::as_feature_path_text(&path)}
			</div>
		</>};
	};
	let action_block = match &feature.action {
		None => html! {},
		Some(action) => html! {<>

			<div class="property">
				<strong>{"Action Type:"}</strong>
				<span>{action.activation_kind}</span>
			</div>

			{action.limited_uses.as_ref().map(|limited_uses| {
				UsesCounter { state: state.clone(), limited_uses }.to_html()
			}).unwrap_or_default()}

			{(!action.conditions_to_apply.is_empty()).then(|| {
				let conditions_to_apply = Arc::new(action.conditions_to_apply.iter().filter_map(|indirect| {
					indirect.resolve(&system).cloned()
				}).collect::<Vec<_>>());

				html! {
					<div class="conditions">
						<h5>{"Conditions Applied on Use"}</h5>
						{conditions_to_apply.iter().map(|condition| {
							html! {<div>
								<h6>{condition.name.clone()}</h6>
								{condition.description.clone()}
								<div>
									<strong>{"Effects:"}</strong>
									<div class="mx-2">
										{mutator_list(&condition.mutators, true)}
									</div>
								</div>
								{condition.criteria.as_ref().map(|evaluator| {
									html! {
										<div>
											<strong class="me-2">{"Only If:"}</strong>
											{format!("TODO: missing description for {:?}", evaluator)}
										</div>
									}
								})}
							</div>}
						}).collect::<Vec<_>>()}
					</div>
				}
			}).unwrap_or_default()}

		</>},
	};
	// TODO: Display mutators
	// TODO: Display criteria
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{feature.name.clone()}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{action_block}
			{description(&feature.description, false)}
		</div>
	</>}
}
