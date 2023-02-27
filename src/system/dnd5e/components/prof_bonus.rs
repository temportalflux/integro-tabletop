use crate::{
	components::{modal, AnnotatedNumber, AnnotatedNumberCard},
	system::dnd5e::{components::SharedCharacter, data::proficiency},
};
use yew::prelude::*;

static TEXT: &'static str = "\
Characters have a proficiency bonus determined by level. Monsters also have this bonus, which is incorporated in their stat blocks. \
The bonus is used in the rules on ability checks, saving throws, and attack rolls.

Your proficiency bonus can't be added to a single die roll or other number more than once. \
For example, if two different rules say you can add your proficiency bonus to a Wisdom saving throw, \
you nevertheless add the bonus only once when you make the save.

Occasionally, your proficiency bonus might be multiplied or divided (doubled or halved, for example) before you apply it. \
For example, the rogue's Expertise feature doubles the proficiency bonus for certain ability checks. \
If a circumstance suggests that your proficiency bonus applies more than once to the same roll, \
you still add it only once and multiply or divide it only once.

By the same token, if a feature or effect allows you to multiply your proficiency bonus when making an ability check \
that wouldn't normally benefit from your proficiency bonus, you still don't add the bonus to the check. \
For that check your proficiency bonus is 0, given the fact that multiplying 0 by any number is still 0. \
For instance, if you lack proficiency in the History skill, you gain no benefit from a feature that lets you \
double your proficiency bonus when you make Intelligence (History) checks.

In general, you don't multiply your proficiency bonus for attack rolls or saving throws. \
If a feature or effect allows you to do so, these same rules apply.";

#[function_component]
pub fn ProfBonus() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let on_click = modal_dispatcher.callback({
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<>
					<div class="modal-header">
						<h1 class="modal-title fs-4">{"Proficiency Bonus"}</h1>
						<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
					</div>
					<div class="modal-body">
						<div class="text-center" style="margin-bottom: 10px;">
							<table class="table table-compact table-striped m-0">
								<thead>
									<tr>
										<th scope="col">{"Charcter Level"}</th>
										<th scope="col">{"Bonus"}</th>
									</tr>
								</thead>
								<tbody>
									{proficiency::level_map().iter().map(|(min, max, bonus)| html! {
										<tr>
											<td>{match (*min, *max) {
												(min, Some(max)) => html! {<span>{min}{"-"}{max}</span>},
												(min, None) => html! {<>{min}{"+"}</>},
											}}</td>
											<td>{"+"}{*bonus}</td>
										</tr>
									}).collect::<Vec<_>>()}
								</tbody>
							</table>
						</div>
						<div style="white-space: pre-line;">
							{TEXT}
						</div>
					</div>
				</>},
				..Default::default()
			})
		}
	});
	html! {
		<AnnotatedNumberCard header={"Proficiency"} footer={"Bonus"} {on_click}>
			<AnnotatedNumber value={state.proficiency_bonus()} show_sign={true} />
		</AnnotatedNumberCard>
	}
}
