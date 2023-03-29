use yew::prelude::*;

use crate::{
	bootstrap::components::Tooltip,
	components::{Tag, Tags},
	system::dnd5e::{components::SharedCharacter, data::mutator::Defense},
	utility::Evaluator,
};

#[function_component]
pub fn DefensesCard() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let defenses = state
		.defenses()
		.iter()
		.fold(Vec::new(), |all, (kind, targets)| {
			targets.iter().fold(all, |mut all, entry| {
				let tooltip = crate::data::as_feature_path_text(&entry.source);
				let damage_type = match &entry.damage_type {
					Some(value) => {
						let damage_type = value.evaluate(&state);
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
						<Tag>
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

fn defence_to_html(defence: Defense) -> Html {
	let style = "width: 12px; height: 12px;".to_owned();
	match defence {
		Defense::Resistance => html! {
			<svg {style} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#00c680" d="M21.18969,15.5h-4.12v7.44h4.12a3.68142,3.68142,0,0,0,2.79-.97,3.75732,3.75732,0,0,0,.94-2.73,3.81933,3.81933,0,0,0-.95-2.74A3.638,3.638,0,0,0,21.18969,15.5Z"></path>
				<path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-8.11,29.51h-6.97l-4.77-9.56h-3.53v9.56h-6.51V10.49h10.63c3.2,0,5.71.71,7.51,2.13a7.21618,7.21618,0,0,1,2.71,6.03,8.78153,8.78153,0,0,1-1.14,4.67005,8.14932,8.14932,0,0,1-3.57,3l5.64,10.91Z"></path>
			</svg>
		},
		Defense::Immunity => html! {
			<svg {style} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-16.75,29.42h-6.5V10.4h6.5Z"></path>
			</svg>
		},
		Defense::Vulnerability => html! {
			<svg {style} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48">
				<path fill="#e40712" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-16.63,30.42h-7.12l-9.02-27.02h7.22L20.2597,31.07l5.38-19.67h7.27Z"></path>
			</svg>
		},
	}
}
