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
			<div class="row" style="margin-top: 0;">
				<div class="col-md-auto">

					<ability::ScoreContainer />
					<ability::SavingThrowContainer />
					<div id="proficiencies-container" class="card" style="border-color: var(--theme-frame-color);">
						<div class="card-body">
							<h5 class="card-title text-center">{"Proficiencies"}</h5>
							{"TODO"}
						</div>
					</div>

				</div>
				<div class="col-md-auto">

					<div class="card" style="width: 90px;">
						<div class="card-body text-center" style="padding: 5px 5px;">
							<h6 class="card-title" style="font-size: 0.8rem;">{"Proficiency"}</h6>
							<div style="font-size: 26px; font-weight: 500;">
								<components::AnnotatedNumber value={3} show_sign={true} />
							</div>
							<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Bonus"}</h6>
						</div>
					</div>
					<div id="skills-container" class="card" style="width: 380px; border-color: var(--theme-frame-color);">
						<div class="card-body">
							<ability::SkillTable />
						</div>
					</div>

				</div>
				<div class="col">
					<div class="row">
						<div class="col-md-auto">
							<div class="card" style="width: 120px;">
								<div class="card-body text-center" style="padding: 5px 5px;">
									<h6 class="card-title" style="font-size: 0.8rem;">{"Speed & Senses"}</h6>
									{"TODO"}
								</div>
							</div>
						</div>
						<div class="col-md-auto">
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
						<div class="col-md-auto">
							<div class="card" style="width: 90px;">
								<div class="card-body text-center" style="padding: 5px 5px;">
									<h6 class="card-title" style="font-size: 0.8rem;">{"Initiative"}</h6>
									<div style="font-size: 26px; font-weight: 500;">
										<components::AnnotatedNumber value={1} show_sign={true} />
									</div>
									<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{"Bonus"}</h6>
								</div>
							</div>
						</div>
						<div class="col-md-auto">
							{"TODO HP"}
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
