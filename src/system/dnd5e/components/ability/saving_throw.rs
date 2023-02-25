use crate::{
	bootstrap::components::Tooltip,
	data::ContextMut,
	system::dnd5e::components::roll::Modifier,
	system::dnd5e::data::{character::Character, proficiency, roll, Ability},
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SavingThrowProps {
	pub title: AttrValue,
	pub value: i32,
	pub proficient: bool,
}

#[function_component]
pub fn SavingThrow(
	SavingThrowProps {
		title,
		value,
		proficient,
	}: &SavingThrowProps,
) -> Html {
	let sign = match *value >= 0 {
		true => "+",
		false => "-",
	};
	html! {
		<div class="card" style="border-color: var(--theme-frame-color-muted);">
			<div class="card-body text-center" style="padding: 5px 5px;">
				<div style="display: inline; width: 100%;">
					<span style="font-size: 0.8rem;">
						{proficiency::Level::from(*proficient)}
						<span style="margin-left: 5px; margin-right: 8px;">{title.clone()}</span>
					</span>
					<span style="font-weight: 700; color: var(--theme-roll-modifier);">{sign}{value.abs()}</span>
				</div>
			</div>
		</div>
	}
}

#[function_component]
pub fn SavingThrowContainer() -> Html {
	let state = use_context::<ContextMut<Character>>().unwrap();

	let saving_throw = {
		let state = state.clone();
		move |ability: Ability| {
			let proficiency = &state.saving_throws()[ability].0;
			let modifier = state.ability_modifier(ability, Some(*proficiency.value()));
			let mod_sign = match modifier >= 0 {
				true => "+",
				false => "-",
			};
			// TODO: Tooltip for proficiency sources
			html! {
				<tr>
					<td class="text-center">{*proficiency.value()}</td>
					<td>{ability.abbreviated_name().to_uppercase()}</td>
					<td class="text-center">
						<span style="font-weight: 700; color: var(--theme-roll-modifier);">
							{mod_sign}{modifier.abs()}
						</span>
					</td>
				</tr>
			}
		}
	};
	let modifiers_html = state.saving_throws().iter_modifiers()
		.fold(Vec::new(), |mut html, (ability, modifiers)| {
			for (target, source_path) in modifiers.iter() {
				let source = crate::data::as_feature_path_text(&source_path);
				html.push(html! {
					<Tooltip content={source}>
						<span class="d-inline-flex" aria-label="Advantage" style=" height: 14px; margin-right: 2px; margin-top: -2px; width: 14px; vertical-align: middle;">
							<Modifier value={roll::Modifier::Advantage} />
						</span>
						<span>
							{"on "}{ability.abbreviated_name().to_uppercase()}
							{target.as_ref().map(|target| format!(" against {target}")).unwrap_or_default()}
						</span>
					</Tooltip>
				});
			}
			html
		});

	html! {
		<div id="saving-throw-container" class="card" style="">
			<div class="card-body text-center" style="padding: 5px;">
				<h5 class="card-title" style="font-size: 0.8rem;">{"Saving Throws"}</h5>
				<div class="row" style="--bs-gutter-x: 0; margin: 0 0 10px 0;">
					<div class="col">
						<table class="table table-compact" style="margin-bottom: 0;">
							<tbody>
								{saving_throw(Ability::Strength)}
								{saving_throw(Ability::Dexterity)}
								{saving_throw(Ability::Constitution)}
							</tbody>
						</table>
					</div>
					<div class="col-auto p-0" style="margin: 0 5px;"><div class="vr" style="min-height: 100%;" /></div>
					<div class="col">
						<table class="table table-compact" style="margin-bottom: 0;">
							<tbody>
								{saving_throw(Ability::Intelligence)}
								{saving_throw(Ability::Wisdom)}
								{saving_throw(Ability::Charisma)}
							</tbody>
						</table>
					</div>
				</div>
				<div class="container overflow-hidden text-center" style="display: none; --bs-gutter-x: 0;">
					<SavingThrow title={"STR"} value={-1} proficient={false} />
					<SavingThrow title={"DEX"} value={0} proficient={false} />
					<SavingThrow title={"CON"} value={3} proficient={false} />
					<SavingThrow title={"INT"} value={6} proficient={true} />
					<SavingThrow title={"WIS"} value={4} proficient={true} />
					<SavingThrow title={"CHA"} value={3} proficient={false} />
				</div>
				<div style="font-size: 11px;">
					{modifiers_html}
				</div>
			</div>
		</div>
	}
}
