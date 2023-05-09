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
			<a class="icon forge" onclick={open_editor.reform(|_| ())} />
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
									<panel::Spells />
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
