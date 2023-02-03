use yew::prelude::*;

pub mod components;
pub mod data;
pub mod theme;

#[function_component]
fn App() -> Html {
	use components::ability;
	return html! {<>
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<a class="navbar-brand" href="/">{"Tabletop Tools"}</a>
					<button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navContent" aria-controls="navContent" aria-expanded="false" aria-label="Toggle navigation">
						<span class="navbar-toggler-icon"></span>
					</button>
					<div class="collapse navbar-collapse" id="navContent">
						<ul class="navbar-nav">
							<li class="nav-item">{"My Characters"}</li>
						</ul>
						<ul class="navbar-nav flex-row flex-wrap ms-md-auto">
							<theme::Dropdown />
						</ul>
					</div>
				</div>
			</nav>
		</header>
		<div class="container overflow-hidden" style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
			<div class="row" style="--bs-gutter-x: 10px;">
				<div class="col-md-auto">

					<div class="row" style="margin-top: 0; --bs-gutter-x: 0;">
						<div class="col-auto">
							<ability::ScoreContainer />
						</div>
						<div class="col-auto">
							<ability::SavingThrowContainer />
							<div class="d-grid justify-content-center">
								<div class="card" style="max-width: 90px;">
									<div class="card-body text-center" style="padding: 5px 5px;">
										<h6 class="card-title" style="font-size: 0.8rem;">{"Proficiency"}</h6>
										<div style="font-size: 26px; font-weight: 500;">
											<components::AnnotatedNumber value={3} show_sign={true} />
										</div>
										<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Bonus"}</h6>
									</div>
								</div>
								<div class="card" style="width: 90px;">
									<div class="card-body text-center" style="padding: 5px 5px;">
										<h6 class="card-title" style="font-size: 0.8rem;">{"Initiative"}</h6>
										<div style="font-size: 26px; font-weight: 500;">
											<components::AnnotatedNumber value={1} show_sign={true} />
										</div>
										<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Bonus"}</h6>
									</div>
								</div>
								<div class="card" style="width: 90px;">
									<div class="card-body text-center" style="padding: 5px 5px;">
										<h6 class="card-title" style="font-size: 0.8rem;">{"Armor"}</h6>
										<div style="font-size: 26px; font-weight: 500;">
											<components::AnnotatedNumber value={10} />
										</div>
										<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Class"}</h6>
									</div>
								</div>
							</div>
						</div>
					</div>
					<div id="proficiencies-container" class="card" style="max-width: 250px; margin: 0 auto; border-color: var(--theme-frame-color);">
						<div class="card-body" style="padding: 5px;">
							<h5 class="card-title text-center" style="font-size: 0.8rem;">{"Proficiencies"}</h5>
							<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
								<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Languages"}</h6>
								<span>{"Common, Gnomish, Sylvan, Undercommon"}</span>
							</div>
							<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
								<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Armor"}</h6>
								<span>{"None"}</span>
							</div>
							<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
								<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Weapons"}</h6>
								<span>{"Crossbow, Light, Dagger, Dart, Quarterstaff, Sling"}</span>
							</div>
							<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
								<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Tools"}</h6>
								<span>{"Cartographer's Tools"}</span>
							</div>
						</div>
					</div>

				</div>
				<div class="col-md-auto">
					
					<div class="card" style="min-width: 150px;">
						<div class="card-body" style="padding: 5px 5px;">
							<h6 class="card-title text-center" style="display: none; font-size: 0.8rem;">{"Walking Speed"}</h6>
							<div class="text-center" style="display: none; width: 100%;">
								<span style="position: relative; font-size: 26px; font-weight: 500;">
									<span>{30}</span>
									<span style="position: absolute; bottom: 2px; font-size: 16px; margin-left: 3px;">{"ft"}</span>
								</span>
							</div>
							<div class="row">
								<div class="col">
									<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Speeds"}</h6>
									<span class="d-flex">
										<span class="flex-grow-1">{"Walking"}</span>
										<span class="ps-2">{"30ft"}</span>
									</span>
									<span class="d-flex">
										<span class="flex-grow-1">{"Flying"}</span>
										<span class="ps-2">{"30ft"}</span>
									</span>
									<span class="d-flex">
										<span class="flex-grow-1">{"Burrow"}</span>
										<span class="ps-2">{"10ft"}</span>
									</span>
								</div>
								<div class="col">
									<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Senses"}</h6>
									<span class="d-flex">
										<span class="flex-grow-1">{"Darkvision"}</span>
										<span class="ps-2">{"60ft"}</span>
									</span>
									<span class="d-flex">
										<span class="flex-grow-1">{"Truesight"}</span>
										<span class="ps-2">{"10ft"}</span>
									</span>
								</div>
							</div>
						</div>
					</div>

					<div id="skills-container" class="card" style="min-width: 300px; border-color: var(--theme-frame-color);">
						<div class="card-body" style="padding: 5px 5px;">
							<ability::SkillTable />
						</div>
					</div>

				</div>
				<div class="col">

					<div class="card" style="min-width: 270px; max-width: 270px;">
						<div class="card-body" style="padding: 5px 5px;">
							<div class="d-flex">
								<div class="flex-grow-1">
									<h5 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color); margin: 0 0 7px 0;">{"Hit Points"}</h5>
									<div class="row text-center m-0" style="--bs-gutter-x: 0;">
										<div class="col" style="min-width: 50px;">
											<div style="font-size: 0.75rem; padding: 0 5px;">{"Current"}</div>
											<div style="font-size: 26px; font-weight: 500;">{"000"}</div>
										</div>
										<div class="col-auto">
											<div style="min-height: 1.2rem;"></div>
											<div style="font-size: 23px; font-weight: 300;">{"/"}</div>
										</div>
										<div class="col" style="min-width: 50px;">
											<div style="font-size: 0.75rem; padding: 0 5px;">{"Max"}</div>
											<div style="font-size: 26px; font-weight: 500;">{"000"}</div>
										</div>
										<div class="col" style="min-width: 50px; margin: 0 5px;">
											<div style="font-size: 0.75rem;">{"Temp"}</div>
											<div style="font-size: 26px; font-weight: 300;">{"000"}</div>
										</div>
									</div>
								</div>
								<div style="width: 80px;">
									<button type="button" class="btn btn-success" style="vertical-align: top; width: 100%; --bs-btn-padding-y: 0px; --bs-btn-font-size: .75rem;">{"Heal"}</button>
									<input type="text" class="form-control text-center" id="hp-amount" style="padding: 0; margin: 0 0 4px 0;" />
									<button type="button" class="btn btn-danger" style="vertical-align: top; width: 100%; --bs-btn-padding-y: 0px; --bs-btn-font-size: .75rem;">{"Damage"}</button>
								</div>
							</div>
							<div class="row m-0 pt-2" style="--bs-gutter-x: 0;">
								<div class="col">
									<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Defences"}</h6>
									<div>
										<span>
											<svg style="width: 12px; height: 12px; margin-top: -4px;" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40.89941 48"><path fill="#00c680" d="M21.18969,15.5h-4.12v7.44h4.12a3.68142,3.68142,0,0,0,2.79-.97,3.75732,3.75732,0,0,0,.94-2.73,3.81933,3.81933,0,0,0-.95-2.74A3.638,3.638,0,0,0,21.18969,15.5Z"></path><path fill="#00c680" d="M40.4497,8c-11,0-20-6-20-8,0,2-9,8-20,8-4,35,20,40,20,40S44.4497,43,40.4497,8Zm-8.11,29.51h-6.97l-4.77-9.56h-3.53v9.56h-6.51V10.49h10.63c3.2,0,5.71.71,7.51,2.13a7.21618,7.21618,0,0,1,2.71,6.03,8.78153,8.78153,0,0,1-1.14,4.67005,8.14932,8.14932,0,0,1-3.57,3l5.64,10.91Z"></path></svg>
											<span style="margin-left: 5px;">{"Cold"}</span>
										</span>
									</div>
								</div>
								<div class="col-auto p-0"><div class="vr" style="min-height: 100%;" /></div>
								<div class="col">
									<h6 class="text-center" style="font-size: 0.8rem; color: var(--bs-card-title-color);">{"Conditions"}</h6>
								</div>
							</div>
						</div>
					</div>

					{"TODO Actions/Inventory/etc"}
				</div>
			</div>
		</div>
	</>};
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<App>::new().render();
}
