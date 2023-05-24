use super::SharedCharacter;
use crate::{
	components::stop_propagation,
	system::dnd5e::data::{action::LimitedUses, character::Persistent},
	utility::{Evaluator, InputExt},
};
use std::sync::Arc;
use yew::prelude::*;

pub struct UsesCounter<'parent> {
	pub state: SharedCharacter,
	pub limited_uses: &'parent LimitedUses,
}

impl<'parent> UsesCounter<'parent> {
	pub fn to_html(self) -> Html {
		let consumed_uses = self.limited_uses.get_uses_consumed(&self.state);
		let max_uses = self.limited_uses.max_uses.evaluate(&self.state);
		if max_uses < 0 {
			return html! {};
		}

		let consume_delta = Callback::from({
			let state = self.state.clone();
			let uses_path = Arc::new(self.limited_uses.get_uses_path());
			move |delta: i32| {
				let new_uses = (consumed_uses as i32).saturating_add(delta).max(0) as u32;
				let uses_path = uses_path.clone();
				state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
					persistent.set_selected_value(uses_path.as_path(), new_uses.to_string());
					None
				}));
			}
		});

		let counter = match max_uses {
			..=0 => Html::default(),
			// we use checkboxes for anything <= 5 max uses
			1..=5 => {
				let toggle_use = Callback::from({
					let consume_delta = consume_delta.clone();
					move |evt: web_sys::Event| {
						let Some(consume_use) = evt.input_checked() else { return; };
						consume_delta.emit(consume_use.then_some(1).unwrap_or(-1));
					}
				});

				html! {<>
					{(0..max_uses as u32)
						.map(|idx| {
							html! {
								<input
									class={"form-check-input slot"} type={"checkbox"}
									checked={idx < consumed_uses}
									onclick={stop_propagation()}
									onchange={toggle_use.clone()}
								/>
							}
						})
						.collect::<Vec<_>>()}
				</>}
			}
			// otherwise we use a numerical counter form
			_ => {
				html! {<UseCounterDelta max_uses={max_uses as u32} {consumed_uses} on_apply={consume_delta.reform(|delta: i32| -delta)} />}
			}
		};

		let reset_desc = self
			.limited_uses
			.reset_on
			.as_ref()
			.map(|rest| format!(" (reset on {:?} Rest)", rest))
			.unwrap_or_default();

		html! {
			<span class="uses d-flex align-items-center">
				<strong class="me-2">{format!("Uses{reset_desc}: ")}</strong>
				{counter}
			</span>
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
struct UseCounterDeltaProps {
	max_uses: u32,
	consumed_uses: u32,
	on_apply: Callback<i32>,
}

#[function_component]
fn UseCounterDelta(
	UseCounterDeltaProps {
		max_uses,
		consumed_uses,
		on_apply,
	}: &UseCounterDeltaProps,
) -> Html {
	let delta_state = use_state_eq(|| 0);

	let clear_delta = Callback::from({
		let delta_state = delta_state.clone();
		move |_| {
			delta_state.set(0);
		}
	});

	let apply_delta = Callback::from({
		let delta_state = delta_state.clone();
		let on_apply = on_apply.clone();
		move |_| {
			if *delta_state != 0 {
				let delta = *delta_state;
				delta_state.set(0);
				on_apply.emit(delta);
			}
		}
	});

	let onclick_sub = Callback::from({
		let delta_state = delta_state.clone();
		move |_| {
			delta_state.set(delta_state.saturating_sub(1));
		}
	});

	let onclick_add = Callback::from({
		let delta_state = delta_state.clone();
		move |_| {
			delta_state.set(delta_state.saturating_add(1));
		}
	});

	let delta_section = match *delta_state {
		0 => Html::default(),
		_ => html! {<>
			<span class="delta">{*delta_state}</span>
			<button type="button" class="btn btn-xs btn-theme" onclick={apply_delta}>{"Apply"}</button>
			<button type="button" class="btn btn-xs btn-theme" onclick={clear_delta}>{"Clear"}</button>
		</>},
	};

	let uses_remaining = max_uses - consumed_uses;
	let new_uses_remaining = (uses_remaining as i32).saturating_add(*delta_state).max(0) as u32;
	html! {<span class="deltaform d-flex align-items-center" onclick={stop_propagation()}>
		<button type="button" class="btn btn-theme sub" onclick={onclick_sub} disabled={new_uses_remaining == 0} />
		<span class="amount">{format!("{new_uses_remaining} / {max_uses}")}</span>
		<button type="button" class="btn btn-theme add" onclick={onclick_add} disabled={new_uses_remaining >= *max_uses} />
		{delta_section}
	</span>}
}
