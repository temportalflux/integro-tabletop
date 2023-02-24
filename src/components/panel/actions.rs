use crate::{
	components::*,
	data::ContextMut,
	system::dnd5e::{
		action::{AttackCheckKind, AttackKindValue},
		character::Character,
		evaluator::Evaluator,
	},
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
	let state = use_context::<ContextMut<Character>>().unwrap();
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
									let (roll, bonus) = &attack.damage_roll;
									let bonus = bonus.evaluate(&*state);
									let roll = roll.as_ref().map(|roll| html!{{roll.to_string()}});
									match (roll, bonus) {
										(None, bonus) => html! {{bonus.max(0)}},
										(Some(roll), 0) => html! {{roll}},
										(Some(roll), 1..=i32::MAX) => html! {<>{roll}{" + "}{bonus}</>},
										(Some(roll), i32::MIN..=-1) => html! {<>{roll}{" - "}{bonus.abs()}</>},
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
	html! {<>
		<Tags>{tag_htmls}</Tags>
		<div style="overflow-y: scroll;">
			{panes}
		</div>
	</>}
}
