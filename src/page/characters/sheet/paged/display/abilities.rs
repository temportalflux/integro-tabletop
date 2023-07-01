use crate::system::dnd5e::{components::ability, data::Ability};
use yew::prelude::*;

#[function_component]
pub fn Page() -> Html {
	html! {<>

		<div class="my-1" style="border: var(--bs-border-width) solid var(--theme-frame-color); border-radius: var(--bs-border-radius);">
			<div class="row m-0" style="--bs-gutter-x: 0;">
				<div class="col">
					<ability::Score ability={Ability::Strength} />
				</div>
				<div class="col-auto p-0"><div class="vr" style="min-height: 100%; background-color: var(--bs-gray-700);" /></div>
				<div class="col">
					<ability::Score ability={Ability::Intelligence} />
				</div>
			</div>
			<div class="hr" style="min-width: 100%; border-color: var(--bs-gray-700);" />
			<div class="row m-0" style="--bs-gutter-x: 0;">
				<div class="col">
					<ability::Score ability={Ability::Dexterity} />
				</div>
				<div class="col-auto p-0"><div class="vr" style="min-height: 100%; background-color: var(--bs-gray-700);" /></div>
				<div class="col">
					<ability::Score ability={Ability::Wisdom} />
				</div>
			</div>
			<div class="hr" style="min-width: 100%; border-color: var(--bs-gray-700);" />
			<div class="row m-0" style="--bs-gutter-x: 0;">
				<div class="col">
					<ability::Score ability={Ability::Constitution} />
				</div>
				<div class="col-auto p-0"><div class="vr" style="min-height: 100%; background-color: var(--bs-gray-700);" /></div>
				<div class="col">
					<ability::Score ability={Ability::Charisma} />
				</div>
			</div>
		</div>

		<div class="card my-1" style="border-color: var(--theme-frame-color);">
			<div class="card-body p-2">
				<h6 class="text-center">{"Modifiers"}</h6>
				<div style="font-weight: 500;">{"Ability Checks"}</div>
				<ability::AbilityModifiers />
				<div class="mt-1" style="font-weight: 500;">{"Saving Throws"}</div>
				<ability::SavingThrowModifiers show_none_label={true} />
			</div>
		</div>

		<div id="skills-container" class="card" style="min-width: 320px; border-color: var(--theme-frame-color);">
			<div class="card-body" style="padding: 5px;">
				<ability::SkillTable />
			</div>
		</div>

	</>}
}
