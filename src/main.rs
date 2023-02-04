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
					<Proficiencies />

				</div>
				<div class="col-md-auto">

					<div class="row m-0 justify-content-center">
						<div class="col p-0">
							<AnnotatedNumberCard header={"Proficiency"} footer={"Bonus"}>
								<AnnotatedNumber value={3} show_sign={true} />
							</AnnotatedNumberCard>
						</div>
						<div class="col p-0">
							<AnnotatedNumberCard header={"Initiative"} footer={"Bonus"}>
								<AnnotatedNumber value={1} show_sign={true} />
							</AnnotatedNumberCard>
						</div>
						<div class="col p-0">
							<AnnotatedNumberCard header={"Armor"} footer={"Class"}>
								<AnnotatedNumber value={10} />
							</AnnotatedNumberCard>
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
							<Nav root_classes={"onesheet-tabs"} disp={NavDisplay::Tabs} default_tab_id={"actions"}>
								<TabContent id="actions" title={html! {{"Actions"}}}>
									<panel::Actions />
								</TabContent>
								<TabContent id="spells" title={html! {{"Spells"}}}>
									{"Spells"}
								</TabContent>
								<TabContent id="inventory" title={html! {{"Inventory"}}}>
									{"Inventory"}
								</TabContent>
								<TabContent id="features" title={html! {{"Features & Traits"}}}>
									{"Features & Traits"}
								</TabContent>
								<TabContent id="description" title={html! {{"Description"}}}>
									{"Description"}
								</TabContent>
							</Nav>
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
