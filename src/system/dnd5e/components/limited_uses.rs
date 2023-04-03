use super::SharedCharacter;
use crate::system::dnd5e::data::{action::LimitedUses, character::Persistent};
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct UsesCounter<'parent> {
	pub state: SharedCharacter,
	pub limited_uses: &'parent LimitedUses,
}

impl<'parent> UsesCounter<'parent> {
	pub fn to_html(self) -> Html {
		let consumed_uses = self.limited_uses.get_uses_consumed(&self.state);
		let Some(max_uses) = self.limited_uses.max_uses.evaluate(&self.state) else { return html! {} };

		let toggle_use = Callback::from({
			let state = self.state.clone();
			let uses_path = Arc::new(self.limited_uses.get_uses_path());
			move |evt: web_sys::Event| {
				let Some(target) = evt.target() else { return; };
				let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
				let consume_use = input.checked();

				let diff = consume_use.then_some(1).unwrap_or(-1);
				let new_uses = (consumed_uses as i32).saturating_add(diff).max(0) as u32;
				let uses_path = uses_path.clone();
				state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
					persistent.set_selected_value(uses_path.as_path(), new_uses.to_string());
					None
				}));
			}
		});

		let use_checkboxes = (0..max_uses)
			.map(|idx| {
				html! {
					<input
						class={"form-check-input slot"} type={"checkbox"}
						checked={idx < consumed_uses}
						onclick={Callback::from(|evt: web_sys::MouseEvent| evt.stop_propagation())}
						onchange={toggle_use.clone()}
					/>
				}
			})
			.collect::<Vec<_>>();

		html! {
			<span class="uses d-flex align-items-center">
				<strong class="me-2">{"Uses: "}</strong>
				{use_checkboxes}
				{match &self.limited_uses.reset_on {
					Some(rest) => html! { <span class="ms-2">{format!("(reset on {:?} Rest)", rest)}</span> },
					None => html! {},
				}}
			</span>
		}
	}
}
