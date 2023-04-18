use crate::{
	components::{Nav, NavDisplay, TabContent},
	system::dnd5e::{
		components::{
			ability, panel, ArmorClass, ConditionsCard, DefensesCard, HitPointMgmtCard,
			InitiativeBonus, Inspiration, ProfBonus, Proficiencies, SpeedAndSenses,
		},
		data::Ability,
	},
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SheetDisplayProps {
	pub open_editor: Callback<()>,
}

#[function_component]
pub fn SheetDisplay(SheetDisplayProps { open_editor }: &SheetDisplayProps) -> Html {
	let floating_editor_btn = html! {
		<div class="ms-auto">
			<a class="forge-icon" onclick={open_editor.reform(|_| ())}>
				<svg class="mt-1" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 46.82 38.62">
					<path fill="#fff" d="M46.17,16.56v4.51s-8.16-.94-12,2.56-3.32,6.47-2.11,7.43,4.17,1.27,4.17,1.27l2.48,3.2v3.08H32.81S31,36.08,26.35,36.08s-6.89,2.48-6.89,2.48H14V35.41l2.36-2.9s2.54.12,3.87-1.27.54-5.32-1.45-5.32H15.35v-.66S5.74,25.44,0,18.31H18V16.56Z"></path>
					<path fill="#fff" d="M27.49,0a52.76,52.76,0,0,0-4.33,11.7,13,13,0,0,0,5.25,2.07,41.51,41.51,0,0,0,4.4-11.55A12.5,12.5,0,0,0,27.49,0Z"></path>
					<path fill="#fff" d="M32.74,6.78a8.53,8.53,0,0,1-1.14,3l13.52,4.79c.57.2,1.32-.69,1.45-1s.62-1.62-.27-1.93Z"></path>
					<path fill="#fff" d="M24.63,3.51A28.29,28.29,0,0,0,23.36,6.7a1.64,1.64,0,0,1-.42-2.14A1.63,1.63,0,0,1,24.63,3.51Z"></path>
					<path fill="#fff" d="M21.76,12,18.94,2.57l-.22,8.23-3.42-4,1.93,5.69L11.15,9.86l7.07,5.36h9.63l-1.73-.44a11.47,11.47,0,0,1-3.83-1.85Z"></path>
				</svg>
			</a>
		</div>
	};
	html! {
		<div class="container overflow-hidden">
			<div class="d-flex">
				{floating_editor_btn}
			</div>
			<div class="row" style="--bs-gutter-x: 10px;">
				<div class="col-md-auto" style="max-width: 210px;">

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
							<HitPointMgmtCard />
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
	}
}
