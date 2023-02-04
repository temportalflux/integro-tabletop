use yew::prelude::*;

pub mod components;
pub mod data;
pub mod theme;

#[function_component]
fn App() -> Html {
	use components::*;
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

					<ability::ScoreContainer />
					<ability::SavingThrowContainer />

					<div id="proficiencies-container" class="card" style="max-width: 200px; margin: 0 auto; border-color: var(--theme-frame-color);">
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

					<div class="row m-0 justify-content-center">
						<div class="col p-0">
							<div class="card m-2">
								<div class="card-body text-center" style="padding: 5px 5px;">
									<h6 class="card-title" style="font-size: 0.8rem;">{"Proficiency"}</h6>
									<div style="font-size: 26px; font-weight: 500; margin: -8px 0;">
										<components::AnnotatedNumber value={3} show_sign={true} />
									</div>
									<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Bonus"}</h6>
								</div>
							</div>
						</div>
						<div class="col p-0">
							<div class="card m-2">
								<div class="card-body text-center" style="padding: 5px 5px;">
									<h6 class="card-title" style="font-size: 0.8rem;">{"Initiative"}</h6>
									<div style="font-size: 26px; font-weight: 500; margin: -8px 0;">
										<components::AnnotatedNumber value={1} show_sign={true} />
									</div>
									<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Bonus"}</h6>
								</div>
							</div>
						</div>
						<div class="col p-0">
							<div class="card m-2">
								<div class="card-body text-center" style="padding: 5px 5px;">
									<h6 class="card-title" style="font-size: 0.8rem;">{"Armor"}</h6>
									<div style="font-size: 26px; font-weight: 500; margin: -8px 0;">
										<components::AnnotatedNumber value={10} />
									</div>
									<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Class"}</h6>
								</div>
							</div>
						</div>
					</div>

					<div id="skills-container" class="card" style="min-width: 300px; border-color: var(--theme-frame-color);">
						<div class="card-body" style="padding: 5px;">
							<ability::SkillTable />
						</div>
					</div>

				</div>
				<div class="col">
					<div class="row m-0" style="--bs-gutter-x: 0;">
						<div class="col">
							{"TODO: Inspiration"}
							<SpeedAndSenses />
						</div>
						<div class="col-auto">
							<HitPoints />
						</div>
					</div>

					<div class="card m-2" style="height: 550px;">
						<div class="card-body" style="padding: 5px;">
							{"TODO Actions/Inventory/etc"}
						</div>
					</div>
				</div>
			</div>
		</div>
	</>};
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<App>::new().render();
}
