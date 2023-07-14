use crate::{
	components::stop_propagation,
	page::characters::sheet::CharacterHandle,
	page::characters::sheet::MutatorImpact,
	system::dnd5e::data::{action::LimitedUses, character::Persistent},
	utility::InputExt,
};
use std::{path::PathBuf, sync::Arc};
use yew::prelude::*;

pub struct UsesCounter<'parent> {
	pub state: CharacterHandle,
	pub limited_uses: &'parent LimitedUses,
}

impl<'parent> UsesCounter<'parent> {
	pub fn to_html(self) -> Html {
		let consumed_uses = self.limited_uses.get_uses_consumed(&self.state) as i32;
		let max_uses = self.limited_uses.get_max_uses(&self.state);
		if max_uses < 0 {
			return html! {};
		}

		let data_path = self.limited_uses.get_uses_path(&self.state);
		let consume_delta = match &data_path {
			None => Callback::default(),
			Some(data_path) => Callback::from({
				let state = self.state.clone();
				let uses_path = Arc::new(data_path.clone());
				move |delta: i32| {
					let new_uses = consumed_uses.saturating_add(delta).max(0) as u32;
					let uses_path = uses_path.clone();
					state.dispatch(Box::new(move |persistent: &mut Persistent| {
						persistent.set_selected_value(uses_path.as_path(), new_uses.to_string());
						MutatorImpact::None
					}));
				}
			}),
		};

		let reset_desc = self
			.limited_uses
			.get_reset_rest(&self.state)
			.map(|rest| format!(" (reset on {:?} Rest)", rest))
			.unwrap_or_default();

		if let Some((cost, resource)) = self.limited_uses.as_consumer() {
			// LimitedUses which are Consumers render differently.
			// Instead of showing the counter, we show a button which tells the user how many uses are consumed.
			let can_use = consumed_uses < max_uses;
			let consume_uses = consume_delta.reform(move |evt: MouseEvent| {
				evt.stop_propagation();
				cost as i32
			});
			html! {
				<div class="uses consumer mt-1">
					<div class="d-flex align-items-center">
						<button
							type="button" class="btn btn-xs btn-theme"
							onclick={consume_uses} disabled={!can_use}
						>{"Use"}</button>
						<span class="ms-2">{format!("Cost: {cost}")}</span>
					</div>
					<div class="d-flex align-items-center mt-1">
						<span class="source-path-sm">{crate::data::as_feature_path_text(resource)}</span>
						<span class="ms-2" style="font-size: 10px;">
							{format!("{} / {max_uses} uses remaining{reset_desc}", max_uses - consumed_uses)}
						</span>
					</div>
				</div>
			}
		} else {
			html! {
				<span class="uses d-flex align-items-center mt-1">
					<strong class="me-2">{format!("Uses{reset_desc}: ")}</strong>
					<ConsumedUsesInput
						max_uses={max_uses as u32}
						consumed_uses={consumed_uses as u32}
						data_path={data_path.clone()}
						can_remove_consumed_uses={true}
					/>
				</span>
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct ConsumedUsesInputProps {
	pub max_uses: u32,
	pub consumed_uses: u32,
	pub data_path: Option<PathBuf>,
	pub can_remove_consumed_uses: bool,
	#[prop_or_default]
	/// If None, the element will automatically
	/// remove the consumed uses from the selector at `data_path`.
	pub consume_delta: Option<Callback</*delta to consumed uses*/ i32>>,
}
#[function_component]
pub fn ConsumedUsesInput(props: &ConsumedUsesInputProps) -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let max_uses = props.max_uses;
	let consumed_uses = props.consumed_uses;

	let consume_delta = match &props.data_path {
		None => Callback::default(),
		Some(data_path) => {
			let data_path = Arc::new(data_path.clone());
			match &props.consume_delta {
				Some(callback) => callback.clone(),
				None => Callback::from({
					let state = state.clone();
					move |delta: i32| {
						let new_uses = (consumed_uses as i32).saturating_add(delta).max(0) as u32;
						let data_path = data_path.clone();
						state.dispatch(Box::new(move |persistent: &mut Persistent| {
							persistent.set_selected_value(data_path.as_path(), new_uses.to_string());
							MutatorImpact::None
						}));
					}
				}),
			}
		}
	};

	match max_uses {
		0 => Html::default(),
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
				{(0..max_uses)
					.map(|idx| {
						html! {
							<input
								class={"form-check-input slot"} type={"checkbox"}
								checked={idx < consumed_uses}
								disabled={!props.can_remove_consumed_uses && idx < consumed_uses}
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
			html! {
				<UseCounterDelta
					{max_uses}
					{consumed_uses}
					can_remove_consumed_uses={props.can_remove_consumed_uses}
					show_apply={props.consume_delta.is_none()}
					on_apply_or_change={consume_delta.reform(|delta: i32| -delta)}
				/>
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
struct UseCounterDeltaProps {
	max_uses: u32,
	consumed_uses: u32,
	show_apply: bool,
	can_remove_consumed_uses: bool,
	on_apply_or_change: Callback<i32>,
}

#[function_component]
fn UseCounterDelta(
	UseCounterDeltaProps {
		max_uses,
		consumed_uses,
		show_apply,
		can_remove_consumed_uses,
		on_apply_or_change,
	}: &UseCounterDeltaProps,
) -> Html {
	let delta_state = use_state_eq(|| 0);
	let on_change = (!*show_apply).then(|| on_apply_or_change.clone());

	let set_delta = Callback::from({
		let consumed_uses = *consumed_uses as i32;
		let can_remove_consumed_uses = *can_remove_consumed_uses;
		let delta_state = delta_state.clone();
		let on_change = on_change.clone();
		move |value: i32| {
			if !can_remove_consumed_uses && consumed_uses + value > consumed_uses {
				return;
			}
			delta_state.set(value);
			if let Some(on_change) = &on_change {
				on_change.emit(value);
			}
		}
	});


	let clear_delta = Callback::from({
		let set_delta = set_delta.clone();
		move |_| {
			set_delta.emit(0);
		}
	});

	let onclick_sub = Callback::from({
		let delta_state = delta_state.clone();
		let set_delta = set_delta.clone();
		move |_| {
			set_delta.emit(delta_state.saturating_sub(1));
		}
	});

	let onclick_add = Callback::from({
		let delta_state = delta_state.clone();
		let set_delta = set_delta.clone();
		move |_| {
			set_delta.emit(delta_state.saturating_add(1));
		}
	});

	let apply_btn = match show_apply {
		false => Html::default(),
		true => {
			let apply_delta = Callback::from({
				let delta_state = delta_state.clone();
				let on_apply = on_apply_or_change.clone();
				move |_| {
					if *delta_state != 0 {
						let delta = *delta_state;
						delta_state.set(0);
						on_apply.emit(delta);
					}
				}
			});
			html!(<button type="button" class="btn btn-xs btn-theme" onclick={apply_delta}>{"Apply"}</button>)
		}
	};
	let delta_section = match *delta_state {
		0 => Html::default(),
		_ => html! {<>
			<span class="delta">{*delta_state}</span>
			{apply_btn}
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
