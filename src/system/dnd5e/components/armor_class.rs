use crate::{
	components::{modal, AnnotatedNumber, AnnotatedNumberCard},
	page::characters::sheet::CharacterHandle,
	system::dnd5e::data::ArmorClassFormula,
};
use yew::prelude::*;

static TEXT: &'static str = "\
Your Armor Class (AC) represents how well your character avoids being wounded in battle. \
Things that contribute to your AC include the armor you wear, the shield you carry, \
and your Dexterity modifier. Not all characters wear armor or carry shields, however. \
Without armor or a shield, your character's AC equals 10 + their Dexterity modifier.";

// TODO: Rules text for Cover (half, 3/4, and full cover)

#[function_component]
pub fn ArmorClass() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
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
		<AnnotatedNumberCard header={"Armor"} footer={"Class"} {on_click}>
			<AnnotatedNumber value={state.armor_class().evaluate(&*state)} />
		</AnnotatedNumberCard>
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct FormulaProps {
	pub formula: ArmorClassFormula,
}

#[function_component]
pub fn FormulaInline(FormulaProps { formula }: &FormulaProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	html! {<>
		<span>{formula.base}</span>
		{formula.bonuses.iter().fold(Vec::new(), |mut html, bounded| {
			let bonus = bounded.evaluate(&state);
			let min = bounded.min.map(|min| format!("min {min}"));
			let max = bounded.max.map(|max| format!("max {max}"));
			html.push(html! {<span>
				{" + "}
				{bounded.ability.abbreviated_name().to_uppercase()}
				{match (min, max) {
					(None, None) => html! {},
					(Some(v), None) | (None, Some(v)) => html! { {format!(" ({v})")} },
					(Some(min), Some(max)) => html! { {format!(" ({min}, {max})")} },
				}}
				{format!(" ({})", bonus)}
			</span>});
			html
		})}
	</>}
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let value = state.armor_class().evaluate(&*state);

	let formula_table = {
		let rows = state
			.armor_class()
			.iter_formulas()
			.map(|(formula, source)| {
				html! {<tr>
					<td>
						<FormulaInline formula={formula.clone()} />
					</td>
					<td>{crate::data::as_feature_path_text(source).unwrap_or_default()}</td>
				</tr>}
			})
			.collect::<Vec<_>>();
		match rows.is_empty() {
			true => html! {},
			false => html! {<div style="margin-bottom: 10px;">
				<table class="table table-compact table-striped m-0">
					<thead>
						<tr class="text-center" style="color: var(--bs-heading-color);">
							<th scope="col">{"Equation"}</th>
							<th scope="col">{"Source"}</th>
						</tr>
					</thead>
					<tbody>
						{rows}
					</tbody>
				</table>
			</div>},
		}
	};
	let bonuses_table = {
		let rows = state
			.armor_class()
			.iter_bonuses()
			.map(|(bonus, source)| {
				html! {<tr>
					<td class="text-center">{match *bonus >= 0 { true => "+", false => "-" }}{bonus.abs()}</td>
					<td>{crate::data::as_feature_path_text(source).unwrap_or_default()}</td>
				</tr>}
			})
			.collect::<Vec<_>>();
		match rows.is_empty() {
			true => html! {},
			false => html! {<div style="margin-bottom: 10px;">
				<table class="table table-compact table-striped m-0">
					<thead>
						<tr class="text-center" style="color: var(--bs-heading-color);">
							<th scope="col">{"Bonus"}</th>
							<th scope="col">{"Source"}</th>
						</tr>
					</thead>
					<tbody>
						{rows}
					</tbody>
				</table>
			</div>},
		}
	};

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{format!("Armor Class ({value})")}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{formula_table}
			{bonuses_table}
			<div class="text-block">
				{TEXT}
			</div>
		</div>
	</>}
}
