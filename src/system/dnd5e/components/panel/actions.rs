use crate::{
	components::*,
	system::dnd5e::{
		components::{SharedCharacter, UsesCounter},
		data::{
			action::{ActivationKind, AttackCheckKind, AttackKindValue},
			DamageRoll,
		},
	},
	utility::Evaluator,
};
use enumset::{EnumSet, EnumSetType};
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
			<table class="table table-compact m-0">
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
		{actions.into_iter().map(|action| {
			html! {<div class="action short">
				<strong>{action.name.clone()}</strong>
				{action.short_desc.as_ref().or(action.description.as_ref()).map(|desc| {
					html! { <div class="text-block">{desc.clone()}</div> }
				}).unwrap_or_default()}
				{action.limited_uses.as_ref().map(|limited_uses| {
					UsesCounter { state: state.clone(), limited_uses }.to_html()
				}).unwrap_or_default()}
			</div>}
		}).collect::<Vec<_>>()}
	</>});

	html! {<>
		<Tags>{tag_htmls}</Tags>
		<div style="overflow-y: scroll; height: 483px;">
			{panes}
		</div>
	</>}
}
