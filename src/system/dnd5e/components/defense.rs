use crate::{
	bootstrap::components::Tooltip,
	components::{modal, Tag, Tags},
	system::dnd5e::{components::SharedCharacter, mutator::Defense},
};
use yew::prelude::*;

static RULES_DESC: &'static str = "Some creatures and objects are exceedingly difficult \
or unusually easy to hurt with certain types of damage.

If a creature or an object has resistance to a damage type, damage of that type \
is halved against it. If a creature or an object has vulnerability to a damage type, \
damage of that type is doubled against it.

Resistance and then vulnerability are applied after all other modifiers to damage. For example, \
a creature has resistance to bludgeoning damage and is hit by an attack that deals \
25 bludgeoning damage. The creature is also within a magical aura that reduces all damage by 5. \
The 25 damage is first reduced by 5 and then halved, so the creature takes 10 damage.

Multiple instances of resistance or vulnerability that affect the same damage type count \
as only one instance. For example, if a creature has resistance to fire damage as well as \
resistance to all nonmagical damage, the damage of a nonmagical fire is reduced by \
half against the creature, not reduced by three-quarters.";

#[function_component]
pub fn DefensesCard() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let onclick = modal_dispatcher.callback(|_| {
		modal::Action::Open(modal::Props {
			centered: true,
			scrollable: true,
			root_classes: classes!("defense"),
			content: html! {<Modal />},
			..Default::default()
		})
	});
	let defenses = state
		.defenses()
		.iter()
		.fold(Vec::new(), |all, (kind, targets)| {
			targets.iter().fold(all, |mut all, entry| {
				let tooltip = crate::data::as_feature_path_text(&entry.source);
				let damage_type = match &entry.damage_type {
					Some(damage_type) => {
						html! {
							<span style="margin-left: 5px;">{damage_type.display_name()}</span>
						}
					}
					None => html! {},
				};
				let context = match &entry.context {
					Some(context) => html! {
						<span style="margin-left: 5px;">{context.clone()}</span>
					},
					None => html! {},
				};
				all.push(html! {
					<Tooltip tag={"span"} style={"margin: 2px;"} content={tooltip} use_html={true}>
						<Tag classes={"defense"}>
							{defence_to_html(kind)}
							{damage_type}
							{context}
						</Tag>
					</Tooltip>
				});
				all
			})
		});
	html! {
		<div class="card m-1" style="height: 85px;" {onclick}>
			<div class="card-body text-center" style="padding: 5px 5px;">
				<h6 class="card-title mb-1" style="font-size: 0.8rem;">{"Defenses"}</h6>
				<div class="d-flex justify-content-center pe-1" style="overflow: scroll; height: 53px;">
					{match defenses.is_empty() {
						true => html! { "None" },
						false => html! {<Tags> {defenses} </Tags>},
					}}
				</div>
			</div>
		</div>
	}
}

fn defence_to_html(defence: Defense) -> Html {
	match defence {
		Defense::Resistance => html! {
			<svg class="defense-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#00c680" d="M21.18969,15.5h-4.12v7.44h4.12a3.68142,3.68142,0,0,0,2.79-.97,3.75732,3.75732,0,0,0,.94-2.73,3.81933,3.81933,0,0,0-.95-2.74A3.638,3.638,0,0,0,21.18969,15.5Z"></path>
				<path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-8.11,29.51h-6.97l-4.77-9.56h-3.53v9.56h-6.51V10.49h10.63c3.2,0,5.71.71,7.51,2.13a7.21618,7.21618,0,0,1,2.71,6.03,8.78153,8.78153,0,0,1-1.14,4.67005,8.14932,8.14932,0,0,1-3.57,3l5.64,10.91Z"></path>
			</svg>
		},
		Defense::Immunity => html! {
			<svg class="defense-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-16.75,29.42h-6.5V10.4h6.5Z"></path>
			</svg>
		},
		Defense::Vulnerability => html! {
			<svg class="defense-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#e40712" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-16.63,30.42h-7.12l-9.02-27.02h7.22L20.2597,31.07l5.38-19.67h7.27Z"></path>
			</svg>
		},
	}
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let mut sections = Vec::new();
	for (defense, entries) in state.defenses().iter() {
		if entries.is_empty() {
			continue;
		}
		sections.push(html! {
			<div class="defense-section">
				<h4>{defence_to_html(defense)}{defense.to_string()}</h4>
				<table class="table table-compact table-striped mx-auto">
					<thead>
						<tr class="text-center" style="color: var(--bs-heading-color);">
							<th scope="col">{"Damage Type"}</th>
							<th scope="col">{"Context"}</th>
							<th scope="col">{"Source"}</th>
						</tr>
					</thead>
					<tbody>
						{entries.iter().map(|entry| {
							let damage_type = match &entry.damage_type {
								None => "All",
								Some(damage_type) => damage_type.display_name(),
							};
							html! {
								<tr>
									<td class="text-center">{damage_type}</td>
									<td class="text-center">{entry.context.clone().unwrap_or_default()}</td>
									<td>{crate::data::as_feature_path_text(&entry.source)}</td>
								</tr>
							}
						}).collect::<Vec<_>>()}
					</tbody>
				</table>
			</div>
		});
	}
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Defenses"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{sections}
			<div class="text-block">{RULES_DESC}</div>
		</div>
	</>}
}
