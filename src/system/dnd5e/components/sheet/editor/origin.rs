use crate::system::{
	core::SourceId,
	dnd5e::{
		components::SharedCharacter,
		data::{
			character::{ActionEffect, Persistent},
			Lineage,
		},
		DnD5e,
	},
};
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

static HELP_TEXT: &'static str = "Lineages and Upbingings are a replacement for races. \
They offer an expanded set of options around traits and features granted from \
the parents and community your character comes from.";

#[function_component]
pub fn OriginTab() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let use_lineages = use_state_eq(|| true);

	let toggle_lineages = Callback::from({
		let use_lineages = use_lineages.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			use_lineages.set(input.checked());
		}
	});
	let lineages_switch = html! {
		<div class="form-check form-switch m-2">
			<label for="useLineages" class="form-check-label">{"Use Lineages & Upbringings"}</label>
			<input  id="useLineages" class="form-check-input"
				type="checkbox" role="switch"
				checked={*use_lineages}
				onchange={toggle_lineages}
			/>
			<div id="useLineagesHelpBlock" class="form-text">{HELP_TEXT}</div>
		</div>
	};

	let view = use_state_eq(|| SlotView::None);
	let set_slot_value = Callback::from({
		let system = system.clone();
		let state = state.clone();
		let view = view.clone();
		move |(slot, source_id): (Slot, Option<SourceId>)| {
			let system = system.clone();
			let view = view.clone();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				match slot {
					Slot::Lineage1 => {
						let library_value = source_id.map(|id| system.lineages.get(&id)).flatten();
						let slot = &mut persistent.lineages[0];
						if slot.as_ref() == library_value {
							return None;
						}
						*slot = library_value.cloned();
					}
					Slot::Lineage2 => {
						let library_value = source_id.map(|id| system.lineages.get(&id)).flatten();
						let slot = &mut persistent.lineages[1];
						if slot.as_ref() == library_value {
							return None;
						}
						*slot = library_value.cloned();
					}
					Slot::Upbringing => {
						let library_value =
							source_id.map(|id| system.upbringings.get(&id)).flatten();
						let slot = &mut persistent.upbringing;
						if slot.as_ref() == library_value {
							return None;
						}
						*slot = library_value.cloned();
					}
				}
				view.set(SlotView::None);
				Some(ActionEffect::Recompile) // TODO: only recompile when leaving the editor, not on every action
			}));
		}
	});
	let on_slot_action = Callback::from({
		let view = view.clone();
		let set_slot_value = set_slot_value.clone();
		move |(slot, action)| match action {
			SlotAction::Remove => {
				set_slot_value.emit((slot, None));
			}
			SlotAction::Cancel => {
				view.set(SlotView::None);
			}
			SlotAction::Edit => {
				view.set(SlotView::EditContents(slot));
			}
			SlotAction::Select => {
				view.set(SlotView::SelectValue(slot));
			}
		}
	});

	let slot_lineage_1 = html! {<SlotControl
		label={"Lineage"}
		name={Slot::Lineage1.get_value_name(state.persistent())}
		is_selected={view.slot() == Some(Slot::Lineage1)}
		on_click={on_slot_action.reform(|action| (Slot::Lineage1, action))}
	/>};
	let slot_lineage_2 = html! {<SlotControl
		label={"Lineage"}
		name={Slot::Lineage2.get_value_name(state.persistent())}
		is_selected={view.slot() == Some(Slot::Lineage2)}
		on_click={on_slot_action.reform(|action| (Slot::Lineage2, action))}
	/>};
	let slot_upbringing = html! {<SlotControl
		label={"Upbringing"}
		name={Slot::Upbringing.get_value_name(state.persistent())}
		is_selected={view.slot() == Some(Slot::Upbringing)}
		on_click={on_slot_action.reform(|action| (Slot::Upbringing, action))}
	/>};
	let slots = html! {
		<div class="row g-2">
			<div class="col">
				{slot_lineage_1}
			</div>
			<div class="col-auto"><div class="vr" style="min-height: 100%;" /></div>
			<div class="col">
				{slot_lineage_2}
			</div>
			<div class="col-auto"><div class="vr" style="min-height: 100%;" /></div>
			<div class="col">
				{slot_upbringing}
			</div>
		</div>
	};

	// TODO: Next steps
	// - collapse lineage accordion with button to show
	// - figure out how to present selected information (mainly for picking selections)
	let panel_content = match &*view {
		SlotView::None => html! {},
		SlotView::EditContents(slot) => html! { {format!("{slot:?}")} },
		SlotView::SelectValue(slot) => match slot {
			Slot::Lineage1 | Slot::Lineage2 => html! {
				<LineageList
					selected_slot={*slot}
					relevant_slots={HashSet::from([Slot::Lineage1, Slot::Lineage2])}
					on_select={set_slot_value.clone()}
				/>
			},
			Slot::Upbringing => html! {
				<UpbringingList on_select={set_slot_value.clone()} />
			},
		},
	};

	html! {<>
		{lineages_switch}
		{slots}
		{panel_content}
	</>}
}

#[derive(Clone, Copy, PartialEq)]
enum SlotView {
	None,
	SelectValue(Slot),
	EditContents(Slot),
}
impl SlotView {
	fn slot(&self) -> Option<Slot> {
		match self {
			Self::None => None,
			Self::SelectValue(slot) => Some(*slot),
			Self::EditContents(slot) => Some(*slot),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Slot {
	Lineage1,
	Lineage2,
	Upbringing,
}
impl Slot {
	fn get_value_name(&self, persistent: &Persistent) -> Option<AttrValue> {
		match self {
			Self::Lineage1 => persistent.lineages[0].as_ref().map(|v| &v.name),
			Self::Lineage2 => persistent.lineages[1].as_ref().map(|v| &v.name),
			Self::Upbringing => persistent.upbringing.as_ref().map(|v| &v.name),
		}
		.cloned()
		.map(AttrValue::from)
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SlotProps {
	label: AttrValue,
	name: Option<AttrValue>,
	is_selected: bool,
	on_click: Callback<SlotAction>,
}
#[derive(Clone, Copy, PartialEq)]
enum SlotAction {
	/// Close the current view for the slot
	Cancel,
	/// Open the edit view for the slot
	Edit,
	/// Clear the slot's value
	Remove,
	/// Open the select view for the slot
	Select,
}
#[function_component]
fn SlotControl(
	SlotProps {
		label,
		name,
		is_selected,
		on_click,
	}: &SlotProps,
) -> Html {
	html! {
		<div class="d-flex h-100 align-items-center">
			<div class="flex-grow-1">
				<div class="text-center">
					<strong>{label.clone()}</strong>
				</div>
				<div>
					{match name.as_ref() {
						Some(name) => name.as_str(),
						None => "Empty",
					}}
				</div>
			</div>
			<div>
				{match (*is_selected, name.is_some()) {
					(true, _) => html! {
						<button
							role="button" class="btn btn-outline-warning btn-sm my-1 mx-auto d-block"
							onclick={on_click.reform(|_| SlotAction::Cancel)}
						>
							{"Cancel"}
						</button>
					},
					(_, true) => html! {<div>
						<button
							role="button" class="btn btn-outline-primary btn-sm my-1 mx-auto d-block"
							onclick={on_click.reform(|_| SlotAction::Edit)}
						>
							{"Edit"}
						</button>
						<button
							role="button" class="btn btn-outline-danger btn-sm my-1 mx-auto d-block"
							onclick={on_click.reform(|_| SlotAction::Remove)}
						>
							{"Remove"}
						</button>
					</div>},
					(_, false) => html! {
						<button
						role="button" class="btn btn-outline-success btn-sm my-1 mx-auto d-block"
							onclick={on_click.reform(|_| SlotAction::Select)}
						>
							{"Select"}
						</button>
					},
				}}
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct LineageListProps {
	selected_slot: Slot,
	relevant_slots: HashSet<Slot>,
	on_select: Callback<(Slot, Option<SourceId>)>,
}
#[function_component]
fn LineageList(
	LineageListProps {
		selected_slot,
		relevant_slots,
		on_select,
	}: &LineageListProps,
) -> Html {
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let state = use_context::<SharedCharacter>().unwrap();

	let current_value_name = selected_slot.get_value_name(state.persistent());
	let lineage_order = {
		let mut vec = system.lineages.iter().collect::<Vec<_>>();
		vec.sort_by(|(_, a), (_, b)| a.name.partial_cmp(&b.name).unwrap());
		vec
	};
	let other_slots = {
		let mut slots = relevant_slots.clone();
		slots.remove(selected_slot);
		slots
	};
	let is_value_in_slots = |value: &Lineage, slots: &HashSet<Slot>| {
		for slot in slots {
			let slot_value = match slot {
				Slot::Lineage1 => state.persistent().lineages[0].as_ref(),
				Slot::Lineage2 => state.persistent().lineages[1].as_ref(),
				Slot::Upbringing => None,
			};
			if let Some(slot_value) = slot_value {
				if slot_value.name == value.name {
					return true;
				}
			}
		}
		false
	};
	html! {
		<div class="accordion my-2" id="all-entries">
			{lineage_order.into_iter().map(move |(source_id, value)| {
				let is_current_selection = is_value_in_slots(value, &[*selected_slot].into());
				let is_otherwise_selected = is_value_in_slots(value, &other_slots);
				html! {
					<ItemEntry
						parent_collapse={"#all-entries"}
						name={value.name.clone()}
						current_slot_name={current_value_name.clone()}
						{is_current_selection}
						{is_otherwise_selected}
						can_select_again={value.can_select_twice}
						on_select={on_select.reform({
							let target_slot = *selected_slot;
							let source_id = source_id.clone();
							move |_| (target_slot, Some(source_id.clone()))
						})}
					>
						<LineageBody lineage={value.clone()} />
					</ItemEntry>
				}
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct UpbringingListProps {
	on_select: Callback<(Slot, Option<SourceId>)>,
}
#[function_component]
fn UpbringingList(UpbringingListProps { on_select }: &UpbringingListProps) -> Html {
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let state = use_context::<SharedCharacter>().unwrap();

	let current_value_name = Slot::Upbringing.get_value_name(state.persistent());
	let item_order = {
		let mut vec = system.upbringings.iter().collect::<Vec<_>>();
		vec.sort_by(|(_, a), (_, b)| a.name.partial_cmp(&b.name).unwrap());
		vec
	};
	html! {
		<div class="accordion my-2" id="all-entries">
			{item_order.into_iter().map(move |(source_id, value)| {
				let is_current_selection = match &state.persistent().upbringing {
					Some(upbringing) => upbringing.name == value.name,
					None => false,
				};
				html! {
					<ItemEntry
						parent_collapse={"#all-entries"}
						name={value.name.clone()}
						current_slot_name={current_value_name.clone()}
						{is_current_selection}
						is_otherwise_selected={false}
						can_select_again={false}
						on_select={on_select.reform({
							let source_id = source_id.clone();
							move |_| (Slot::Upbringing, Some(source_id.clone()))
						})}
					>
						<div style="white-space: pre-line;">
							{value.description.clone()}
						</div>
					</ItemEntry>
				}
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ItemEntryProps {
	parent_collapse: AttrValue,
	name: AttrValue,
	current_slot_name: Option<AttrValue>,
	is_current_selection: bool,
	is_otherwise_selected: bool,
	can_select_again: bool,
	children: Children,
	on_select: Callback<()>,
}

#[function_component]
fn ItemEntry(
	ItemEntryProps {
		parent_collapse,
		name,
		current_slot_name,
		is_current_selection,
		is_otherwise_selected,
		can_select_again,
		children,
		on_select,
	}: &ItemEntryProps,
) -> Html {
	use convert_case::{Case, Casing};

	let disabled_btn = |text: Html| {
		html! {
			<button type="button" class="btn btn-outline-secondary my-1 w-100" disabled={true}>
				{text}
			</button>
		}
	};
	let select_btn = |bonus_text: Html| {
		let onclick = on_select.reform(|_| ());
		html! {
			<button type="button" class="btn btn-outline-success my-1 w-100" {onclick}>
				{"Select"}{bonus_text}
			</button>
		}
	};
	let replace_btn = |name: &AttrValue| {
		let onclick = on_select.reform(|_| ());
		html! {
			<button type="button" class="btn btn-outline-warning my-1 w-100" {onclick}>
				{"Replace "}{name.clone()}
			</button>
		}
	};

	let slot_buttons = match (
		*is_current_selection,
		current_slot_name,
		*is_otherwise_selected,
		*can_select_again,
	) {
		// is in this slot
		(true, _, _, _) => disabled_btn(html! {{"Currently Selected"}}),
		// option already selected for another slot, and cannot be selected again
		(_, _, true, false) => disabled_btn(html! {{"Cannot Select Again"}}),
		// Slot is empty, and this option is not-yet used
		(_, None, false, _) => select_btn(html! {}),
		// Slot is empty, this option is in another slot, but it can be used again
		(_, None, true, true) => select_btn(html! {{" Again"}}),
		// Slot has a value & it is not this option
		(_, Some(name), _, _) => replace_btn(name),
	};

	let id = name.as_str().to_case(Case::Kebab);
	html! {
		<div class="accordion-item">
			<h2 class="accordion-header">
				<button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target={format!("#{id}")}>
					{name.clone()}
				</button>
			</h2>
			<div {id} class="accordion-collapse collapse" data-bs-parent={parent_collapse.clone()}>
				<div class="accordion-body">
					<div>{slot_buttons}</div>
					{children.clone()}
				</div>
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct LineageBodyProps {
	lineage: Lineage,
}
#[function_component]
fn LineageBody(LineageBodyProps { lineage }: &LineageBodyProps) -> Html {
	html! {<>
		<div style="white-space: pre-line;">
			{lineage.description.clone()}
		</div>
	</>}
}
