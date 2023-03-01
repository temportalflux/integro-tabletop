use crate::{
	components::{modal, Nav, NavDisplay, TabContent},
	system::dnd5e::{
		components::{
			ability, panel, ArmorClass, ConditionsCard, DefensesCard, HitPoints, InitiativeBonus,
			Inspiration, ProfBonus, Proficiencies, SpeedAndSenses,
		},
		data::{
			character::{Character, Persistent},
			Ability,
		},
	},
};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SharedCharacter(UseReducerHandle<Character>);
impl std::ops::Deref for SharedCharacter {
	type Target = UseReducerHandle<Character>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl SharedCharacter {
	pub fn new_dispatch<I, F>(&self, mutator: F) -> Callback<I>
	where
		I: 'static,
		F: Fn(I, &mut Persistent, &std::rc::Rc<Character>) + 'static,
	{
		let handle = self.0.clone();
		let mutator = std::rc::Rc::new(mutator);
		Callback::from(move |input: I| {
			let mutator = mutator.clone();
			handle.dispatch(Box::new(move |a, b| {
				(*mutator)(input, a, b);
			}));
		})
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct CharacterSheetPageProps {
	pub character: Persistent,
}

#[function_component]
pub fn CharacterSheetPage(CharacterSheetPageProps { character }: &CharacterSheetPageProps) -> Html {
	let character = SharedCharacter(use_reducer({
		let character = character.clone();
		move || Character::from(character)
	}));
	let modal_dispatcher = modal::Context::from(use_reducer(|| modal::State::default()));

	html! {
		<ContextProvider<SharedCharacter> context={character.clone()}>
			<ContextProvider<modal::Context> context={modal_dispatcher.clone()}>
				<div style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
					<modal::GeneralPurpose />
					<div class="container overflow-hidden">
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

								<div class="d-flex justify-content-center">
									<SpeedAndSenses />
								</div>

								<div id="skills-container" class="card" style="min-width: 320px; border-color: var(--theme-frame-color);">
									<div class="card-body" style="padding: 5px;">
										<ability::SkillTable />
									</div>
								</div>

							</div>
							<div class="col">
								<div class="row m-0" style="--bs-gutter-x: 0;">
									<div class="col-auto">
										<div class="d-flex align-items-center" style="height: 100%;">
											<InitiativeBonus />
											<ArmorClass />
											<Inspiration />
										</div>
									</div>
									<div class="col">
										<HitPoints />
									</div>
								</div>
								<div class="row m-0" style="--bs-gutter-x: 0;">
									<div class="col-auto">
										<div class="d-flex align-items-center" style="height: 100%;">
											<ProfBonus />
										</div>
									</div>
									<div class="col">
										<DefensesCard />
									</div>
									<div class="col">
										<ConditionsCard />
									</div>
								</div>

								<div class="card m-1" style="height: 550px;">
									<div class="card-body" style="padding: 5px;">
										<Nav root_classes={"onesheet-tabs"} disp={NavDisplay::Tabs} default_tab_id={"features"}>
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
												<panel::Features />
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
				</div>
			</ContextProvider<modal::Context>>
		</ContextProvider<SharedCharacter>>
	}
}
