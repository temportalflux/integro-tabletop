use crate::{
	components::*,
	system::dnd5e::{
		components::{
			editor::{description, mutator_list},
			SharedCharacter, UsesCounter,
		},
		data::{
			action::{Action, ActionSource, ActivationKind, AttackCheckKind, AttackKindValue},
			character::{ActionEffect, Persistent},
			DamageRoll,
		},
		DnD5e,
	},
	utility::Evaluator,
};
use enumset::{EnumSet, EnumSetType};
use std::sync::Arc;
use yew::prelude::*;

#[derive(EnumSetType)]
pub enum ActionTag {
	Attack,
	Action,
	BonusAction,
	Reaction,
	Other,
	LimitedUse,
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
	let mut panes = Vec::new();
	if selected_tags.contains(ActionTag::Attack) {
		let attacks = {
			let mut attacks = state
				.actions()
				.iter()
				.filter_map(|action| match action.attack.as_ref() {
					Some(attack) => Some((action.name.clone(), attack)),
					None => None,
				})
				.collect::<Vec<_>>();
			attacks.sort_by(|(a, _), (b, _)| a.cmp(b));
			attacks
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
					{attacks.into_iter().map(|(name, attack)| {
						html! {
							<tr class="align-middle">
								<td>{name}</td>
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

	let actions = {
		let mut actions = state
			.actions()
			.iter()
			.filter(|action| {
				let mut passes_any = false;
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
				passes_any
			})
			.collect::<Vec<_>>();
		actions.sort_by(|a, b| a.name.cmp(&b.name));
		actions
	};
	panes.push(html! {<>
		{actions.into_iter().cloned().map(|action| html! { <ActionOverview {action} /> }).collect::<Vec<_>>()}
	</>});

	html! {<>
		<Tags>{tag_htmls}</Tags>
		<div style="overflow-y: scroll; height: 483px;">
			{panes}
		</div>
	</>}
}

fn action_source_text(
	source: &Option<ActionSource>,
	inventory: &crate::system::dnd5e::data::item::Inventory,
) -> Option<String> {
	source
		.as_ref()
		.map(|source| crate::data::as_feature_path_text(&source.as_path(inventory)))
		.flatten()
}

#[derive(Clone, PartialEq, Properties)]
struct ActionProps {
	pub action: Action,
}
#[function_component]
fn ActionOverview(ActionProps { action }: &ActionProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let onclick = modal_dispatcher.callback({
		let action = action.clone();
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("action"),
				content: html! {<Modal action={action.clone()} />},
				..Default::default()
			})
		}
	});
	let source_path = action_source_text(&action.source, state.inventory())
		.map(|path_text| {
			html! {
				<span style="color: var(--bs-gray-600);">{" ("}{path_text}{")"}</span>
			}
		})
		.unwrap_or_default();
	html! {
		<div class="action short mb-2 border-bottom-theme-muted" {onclick}>
			<strong class="title">{action.name.clone()}</strong>
			<span class="subtitle">
				{action.activation_kind}
				{source_path}
			</span>
			{description(&action.description, true)}
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
		</div>
	}
}

#[function_component]
fn Modal(ActionProps { action }: &ActionProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{action.name.clone()}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">

			<div class="property">
				<strong>{"Action Type:"}</strong>
				<span>{action.activation_kind}</span>
			</div>
			{action_source_text(&action.source, state.inventory()).map(|path_text| html! {
				<div class="property">
					<strong>{"Source:"}</strong>
					<span>{path_text}</span>
				</div>
			}).unwrap_or_default()}

			{action.limited_uses.as_ref().map(|limited_uses| {
				UsesCounter { state: state.clone(), limited_uses }.to_html()
			}).unwrap_or_default()}

			{description(&action.description, false)}

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
		</div>
	</>}
}
