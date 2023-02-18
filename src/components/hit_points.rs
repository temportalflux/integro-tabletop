use crate::{
	bootstrap::components::Tooltip,
	data::ContextMut,
	system::dnd5e::{character::State, mutator::Defense},
};
use yew::prelude::*;

fn defence_to_html(defence: Defense) -> Html {
	let style = "width: 12px; height: 12px; margin-top: -4px;".to_owned();
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
	let data = use_context::<ContextMut<State>>().unwrap();

	let onclick_heal = data.new_mutator(|character| {
		character.add_hit_points(1);
	});
	let onclick_dmg = data.new_mutator(|character| {
		character.sub_hit_points(1);
	});

	let hit_points = data.hit_points();
	let defenses = data
		.defenses()
		.iter()
		.fold(Vec::new(), |all, (kind, targets)| {
			targets.iter().fold(all, |mut all, (target, sources)| {
				let tooltip = crate::data::as_feature_paths_html(sources.iter());
				all.push(html! {<div>
					<Tooltip tag={"span"} content={tooltip} use_html={true}>
						{defence_to_html(kind)}
						<span style="margin-left: 5px;">{target.clone()}</span>
					</Tooltip>
				</div>});
				all
			})
		});
	let conditions = Vec::<Html>::new();

	html! {
		<div class="card m-2" style="min-width: 270px; max-width: 270px;">
			<div class="card-body" style="padding: 5px 5px;">
				<div class="d-flex">
					<div class="flex-grow-1">
						<h5 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color); margin: 0 0 7px 0;">{"Hit Points"}</h5>
						<div class="row text-center m-0" style="--bs-gutter-x: 0;">
							<div class="col" style="min-width: 50px;">
								<div style="font-size: 0.75rem; padding: 0 5px;">{"Current"}</div>
								<div style="font-size: 26px; font-weight: 500;">{hit_points.0}</div>
							</div>
							<div class="col-auto">
								<div style="min-height: 1.2rem;"></div>
								<div style="font-size: 23px; font-weight: 300;">{"/"}</div>
							</div>
							<div class="col" style="min-width: 50px;">
								<div style="font-size: 0.75rem; padding: 0 5px;">{"Max"}</div>
								<div style="font-size: 26px; font-weight: 500;">{hit_points.1}</div>
							</div>
							<div class="col" style="min-width: 50px; margin: 0 5px;">
								<div style="font-size: 0.75rem;">{"Temp"}</div>
								<div style="font-size: 26px; font-weight: 300;">{hit_points.2}</div>
							</div>
						</div>
					</div>
					<div style="width: 80px;">
						<button type="button" class="btn btn-success" style="vertical-align: top; width: 100%; --bs-btn-padding-y: 0px; --bs-btn-font-size: .75rem;" onclick={onclick_heal}>{"Heal"}</button>
						<input type="text" class="form-control text-center" id="hp-amount" style="padding: 0; margin: 0 0 4px 0;" />
						<button type="button" class="btn btn-danger" style="vertical-align: top; width: 100%; --bs-btn-padding-y: 0px; --bs-btn-font-size: .75rem;" onclick={onclick_dmg}>{"Damage"}</button>
					</div>
				</div>
				<div class="row m-0 pt-2" style="--bs-gutter-x: 0;">
					<div class="col">
						<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Defences"}</h6>
						<div>
							{match defenses.is_empty() {
								true => html! { "None" },
								false => html! {<> {defenses} </>},
							}}
						</div>
					</div>
					<div class="col-auto p-0"><div class="vr" style="min-height: 100%;" /></div>
					<div class="col">
						<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Conditions"}</h6>
						{match conditions.is_empty() {
							true => html! { "None" },
							false => html! {<> {conditions} </>},
						}}
					</div>
				</div>
			</div>
		</div>
	}
}
