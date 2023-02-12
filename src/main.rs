use yew::prelude::*;

pub mod components;
pub mod data;
pub mod system;
pub mod theme;

#[derive(Clone, PartialEq)]
pub struct Compiled<T> {
	wrapped: T,
	update_root_channel: UseStateHandle<()>,
}
impl<T> std::ops::Deref for Compiled<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.wrapped
	}
}
impl<T> Compiled<T> {
	pub fn update_root(&self) {
		self.update_root_channel.set(());
	}
}

#[function_component]
fn App() -> Html {
	let character = system::dnd5e::character::changeling_character();

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
		<CharacterSheetPage {character} />
	</>};
}

#[derive(Clone, PartialEq, Properties)]
struct CharacterSheetPageProps {
	character: system::dnd5e::character::Character,
}

#[function_component]
fn CharacterSheetPage(CharacterSheetPageProps { character }: &CharacterSheetPageProps) -> Html {
	use components::*;
	use data::ContextMut;
	use system::dnd5e::character::State;
	use system::dnd5e::Ability;

	let character = ContextMut::<State>::from(use_reducer({
		let character = character.clone();
		move || State::from(character)
	}));

	html! {
		<ContextProvider<ContextMut<State>> context={character.clone()}>
			<div class="container overflow-hidden" style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
				<div class="row" style="--bs-gutter-x: 10px;">
					<div class="col-md-auto">

						<div class="row m-0" style="--bs-gutter-x: 0;">
							<div class="col">
								<ability::Score ability={Ability::Strength} />
								<ability::Score ability={Ability::Dexterity} />
								<ability::Score ability={Ability::Constitution} />
							</div>
							<div class="col">
								<ability::Score ability={Ability::Intelligence} />
								<ability::Score ability={Ability::Wisdom} />
								<ability::Score ability={Ability::Charisma} />
							</div>
						</div>

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
										<panel::Inventory />
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
		</ContextProvider<ContextMut<State>>>
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<App>::new().render();
}
