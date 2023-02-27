use crate::{
	bootstrap::components::Tooltip,
	components::modal,
	system::dnd5e::components::{roll::Modifier, SharedCharacter},
	system::dnd5e::data::{roll, Ability},
};
use enumset::EnumSet;
use yew::prelude::*;

static TEXT: &'static str = "\
A saving throw — also called a save — represents an attempt to resist a spell, a trap, a poison, \
a disease, or a similar threat. You don't normally decide to make a saving throw; you are forced \
to make one because your character or monster is at risk of harm.

To make a saving throw, roll a d20 and add the appropriate ability modifier. \
For example, you use your Dexterity modifier for a Dexterity saving throw.

A saving throw can be modified by a situational bonus or penalty and can be \
affected by advantage and disadvantage, as determined by the DM.

Each class gives proficiency in at least two saving throws. The wizard, for example, \
is proficient in Intelligence saves. As with skill proficiencies, proficiency in a saving throw \
lets a character add their proficiency bonus to saving throws made using a \
particular ability score. Some monsters have saving throw proficiencies as well.

The Difficulty Class for a saving throw is determined by the effect that causes it. \
For example, the DC for a saving throw allowed by a spell is determined by the caster's \
spellcasting ability and proficiency bonus.

The result of a successful or failed saving throw is also detailed in the effect \
that allows the save. Usually, a successful save means that a creature \
suffers no harm, or reduced harm, from an effect.";

#[derive(Clone, PartialEq, Properties)]
pub struct SavingThrowProps {
	pub ability: Ability,
	pub abbreviated: bool,
}

#[function_component]
pub fn SavingThrow(
	SavingThrowProps {
		ability,
		abbreviated,
	}: &SavingThrowProps,
) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let proficiency = &state.saving_throws().get_prof(*ability);
	let modifier = state.ability_modifier(*ability, Some(*proficiency.value()));
	let mod_sign = match modifier >= 0 {
		true => "+",
		false => "-",
	};
	html! {<tr>
		<Tooltip tag={"td"} classes={"text-center"} use_html={true} content={abbreviated.then(|| {
			crate::data::as_feature_paths_html(proficiency.sources().iter().map(|(path, _)| path))
		}).flatten()}>
			{*proficiency.value()}
		</Tooltip>
		<td class={"text-center"}>{match *abbreviated {
			true => ability.abbreviated_name().to_uppercase(),
			false => ability.long_name().to_owned(),
		}}</td>
		<td class="text-center">
			<span style="font-weight: 700; color: var(--theme-roll-modifier);">
				{mod_sign}{modifier.abs()}
			</span>
		</td>
		{(!abbreviated).then(|| html! {<td>
			{proficiency.sources().iter().filter_map(|(path, _)| {
				crate::data::as_feature_path_text(path)
			}).map(|text| html! {<div>{text}</div>}).collect::<Vec<_>>()}
		</td>}).unwrap_or_default()}
	</tr>}
}

#[function_component]
pub fn SavingThrowContainer() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let on_click = modal_dispatcher.callback({
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<Modal />},
				..Default::default()
			})
		}
	});

	html! {
		<div id="saving-throw-container" class="card" onclick={on_click}>
			<div class="card-body" style="padding: 5px;">
				<h5 class="card-title text-center" style="font-size: 0.8rem;">{"Saving Throws"}</h5>
				<div class="row" style="--bs-gutter-x: 0; margin: 0 0 10px 0;">
					<div class="col">
						<table class="table table-compact" style="margin-bottom: 0;">
							<tbody>
								<SavingThrow ability={Ability::Strength} abbreviated={true} />
								<SavingThrow ability={Ability::Dexterity} abbreviated={true} />
								<SavingThrow ability={Ability::Constitution} abbreviated={true} />
							</tbody>
						</table>
					</div>
					<div class="col-auto p-0" style="margin: 0 5px;"><div class="vr" style="min-height: 100%;" /></div>
					<div class="col">
						<table class="table table-compact" style="margin-bottom: 0;">
							<tbody>
								<SavingThrow ability={Ability::Intelligence} abbreviated={true} />
								<SavingThrow ability={Ability::Wisdom} abbreviated={true} />
								<SavingThrow ability={Ability::Charisma} abbreviated={true} />
							</tbody>
						</table>
					</div>
				</div>
				<div style="font-size: 11px;">
					{state.saving_throws().iter_modifiers().map(|(ability, target, source_path)| {
						let style="height: 14px; margin-right: 2px; margin-top: -2px; width: 14px; vertical-align: middle;";
						html! {
							<Tooltip content={crate::data::as_feature_path_text(&source_path)}>
								<span class="d-inline-flex" aria-label="Advantage" {style}>
									<Modifier value={roll::Modifier::Advantage} />
								</span>
								<span>
									{"on "}{ability.abbreviated_name().to_uppercase()}
									{target.as_ref().map(|target| format!(" against {target}")).unwrap_or_default()}
								</span>
							</Tooltip>
						}
					}).collect::<Vec<_>>()}
				</div>
			</div>
		</div>
	}
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let abilities = {
		let mut all = EnumSet::<Ability>::all().into_iter().collect::<Vec<_>>();
		all.sort();
		all
	};
	let modifiers_section = {
		let modifier_rows = state
			.saving_throws()
			.iter_modifiers()
			.map(|(ability, target, source_path)| {
				let style="height: 14px; margin-right: 2px; margin-top: -2px; width: 14px; vertical-align: middle;";
				html! {
					<tr>
						<td class="text-center">
							<span class="d-inline-flex" aria-label="Advantage" {style}>
								<Modifier value={roll::Modifier::Advantage} />
							</span>
						</td>
						<td class="text-center">{ability.long_name()}</td>
						<td class="text-center">{target.clone().unwrap_or_default()}</td>
						<td>{crate::data::as_feature_path_text(&source_path)}</td>
					</tr>
				}
			})
			.collect::<Vec<_>>();
		if modifier_rows.is_empty() {
			html! {}
		} else {
			html! {<>
				<h5 class="text-center" style="margin-top: 15px;">{"Modifiers"}</h5>
				<table class="table table-compact table-striped m-0">
					<thead>
						<tr class="text-center" style="color: var(--bs-heading-color);">
							<th scope="col">{""}</th>
							<th scope="col">{"Ability"}</th>
							<th scope="col">{"Target"}</th>
							<th scope="col">{"Sources"}</th>
						</tr>
					</thead>
					<tbody>
						{modifier_rows}
					</tbody>
				</table>
			</>}
		}
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Saving Throws"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<table class="table table-compact table-striped m-0">
				<thead>
					<tr class="text-center" style="color: var(--bs-heading-color);">
						<th scope="col">{"Prof"}</th>
						<th scope="col">{"Ability"}</th>
						<th scope="col">{"Bonus"}</th>
						<th scope="col">{"Sources"}</th>
					</tr>
				</thead>
				<tbody>
					{abilities.into_iter().map(|ability| html! {
						<SavingThrow {ability} abbreviated={false} />
					}).collect::<Vec<_>>()}
				</tbody>
			</table>
			{modifiers_section}
			<div style="margin-top: 15px; white-space: pre-line;">
				{TEXT}
			</div>
		</div>
	</>}
}
