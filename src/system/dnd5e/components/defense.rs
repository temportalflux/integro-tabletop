use crate::{
	bootstrap::components::Tooltip,
	components::{modal, Tag, Tags},
	page::characters::sheet::CharacterHandle,
	system::dnd5e::mutator::Defense,
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
	let state = use_context::<CharacterHandle>().unwrap();
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
							<DefenseIcon value={kind} />
							{damage_type}
							{context}
						</Tag>
					</Tooltip>
				});
				all
			})
		});
	html! {
		<div class="card m-1" style="height: 80px;" {onclick}>
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

#[derive(Clone, PartialEq, Properties)]
pub struct GeneralProp<T: Clone + PartialEq> {
	pub value: T,
}
#[function_component]
fn DefenseIcon(props: &GeneralProp<Defense>) -> Html {
	let mut classes = classes!("icon", "defense");
	classes.push(match props.value {
		Defense::Resistance => "resistance",
		Defense::Immunity => "immunity",
		Defense::Vulnerability => "vulnerability",
	});
	html! {
		<span class={classes} />
	}
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let mut sections = Vec::new();
	for (defense, entries) in state.defenses().iter() {
		if entries.is_empty() {
			continue;
		}
		sections.push(html! {
			<div class="defense-section">
				<h4><DefenseIcon value={defense} />{defense.to_string()}</h4>
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
