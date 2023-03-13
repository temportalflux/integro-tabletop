use crate::system::{
	core::SourceId,
	dnd5e::{
		components::SharedCharacter,
		data::character::{ActionEffect, Persistent},
		DnD5e,
	},
};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

static HELP_TEXT: &'static str = "Lineages and Upbingings are a replacement for races. \
They offer an expanded set of options around traits and features granted from \
the parents and community your character comes from.";

#[derive(Clone, Copy, PartialEq)]
enum SelectedSlot {
	None,
	SelectValue(usize),
	EditContents(usize),
}
impl SelectedSlot {
	fn idx(&self) -> Option<usize> {
		match self {
			Self::None => None,
			Self::SelectValue(idx) => Some(*idx),
			Self::EditContents(idx) => Some(*idx),
		}
	}
}

#[function_component]
pub fn OriginTab() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let use_lineages = use_state_eq(|| true);
	let selected_lineage = use_state_eq(|| SelectedSlot::None);

	let toggle_lineages = Callback::from({
		let use_lineages = use_lineages.clone();
		move |evt: web_sys::Event| {
			let Some(target) = evt.target() else { return; };
			let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
			use_lineages.set(input.checked());
		}
	});

	let clear_selected_slot = Callback::from({
		let selected_lineage = selected_lineage.clone();
		move |_| {
			selected_lineage.set(SelectedSlot::None);
		}
	});
	let edit_lineage = Callback::from({
		let selected_lineage = selected_lineage.clone();
		move |slot_idx| {
			selected_lineage.set(SelectedSlot::EditContents(slot_idx));
		}
	});
	let select_lineage = Callback::from({
		let selected_lineage = selected_lineage.clone();
		move |slot_idx| {
			selected_lineage.set(SelectedSlot::SelectValue(slot_idx));
		}
	});
	let on_lineage_selected = Callback::from({
		let system = system.clone();
		let state = state.clone();
		let selected_lineage = selected_lineage.clone();
		move |(slot_idx, source_id): (usize, Option<SourceId>)| {
			let system = system.clone();
			let selected_lineage = selected_lineage.clone();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				let Some(slot) = persistent.lineages.get_mut(slot_idx) else { return None; };
				let new_lineage = source_id.map(|id| system.lineages.get(&id)).flatten();
				if slot.as_ref() == new_lineage {
					return None;
				}
				*slot = new_lineage.cloned();
				selected_lineage.set(SelectedSlot::None);
				Some(ActionEffect::Recompile) // TODO: only recompile when leaving the editor, not on every action
			}));
		}
	});

	let lineage_slots = state
		.persistent()
		.lineages
		.iter()
		.enumerate()
		.map({
			let clear_selected_slot = clear_selected_slot.clone();
			let select_lineage = select_lineage.clone();
			let on_lineage_selected = on_lineage_selected.clone();
			let selected_slot = selected_lineage.clone();
			move |(idx, slot)| {
				let cancel_selection = clear_selected_slot.reform(move |_| ());
				let select_lineage = select_lineage.reform(move |_| idx);
				let remove_selection = on_lineage_selected.reform(move |_| (idx, None));
				let edit_lineage = edit_lineage.reform(move |_| idx);
				html! {
					<div class="row my-1">
						<div class="col-auto my-auto">
							<strong>{"Slot "}{idx + 1}</strong>
						</div>
						<div class="col my-auto">
							{match slot.as_ref() {
								Some(item) => item.name.as_str(),
								None => "Empty",
							}}
						</div>
						<div class="col-3 my-auto">
							<div class="d-flex">
								{match (selected_slot.idx(), slot.is_some()) {
									(Some(selected_idx), _) if selected_idx == idx => html! {
										<button role="button" class="btn btn-outline-warning btn-sm mx-auto"
											onclick={cancel_selection}
										>{"Cancel"}</button>
									},
									(_, true) => html! {<>
										<button role="button" class="btn btn-outline-primary btn-sm mx-auto"
											onclick={edit_lineage}
										>{"Edit"}</button>
										<button
											role="button" class="btn btn-outline-danger btn-sm mx-auto"
											onclick={remove_selection}
										>{"Remove"}</button>
									</>},
									(_, false) => html! {
										<button role="button" class="btn btn-outline-success btn-sm mx-auto"
											onclick={select_lineage}
										>{"Select"}</button>
									},
								}}
							</div>
						</div>
					</div>
				}
			}
		})
		.collect::<Vec<_>>();

	// TODO: Next steps
	// - collapse lineage accordion with button to show
	// - figure out how to present selected information (mainly for picking selections)
	let panel_content = match &*selected_lineage {
		SelectedSlot::None => html! {},
		SelectedSlot::SelectValue(selected_slot) => {
			let lineage_order = {
				let mut vec = system.lineages.iter().collect::<Vec<_>>();
				vec.sort_by(|(_, a), (_, b)| a.name.partial_cmp(&b.name).unwrap());
				vec
			};
			let other_slots = state
				.persistent()
				.lineages
				.iter()
				.enumerate()
				.map(|(idx, _)| idx)
				.filter(|idx| idx != selected_slot)
				.collect::<Vec<_>>();
			let get_slot_value = |slot_idx: usize| {
				state
					.persistent()
					.lineages
					.get(slot_idx)
					.map(Option::as_ref)
					.flatten()
			};
			let current_slot_name = get_slot_value(*selected_slot).map(|value| &value.name);
			let is_lineage_in = |id: &String, list: &[usize]| {
				for slot_idx in list {
					if let Some(lineage) = get_slot_value(*slot_idx) {
						if &lineage.name == id {
							return true;
						}
					}
				}
				false
			};
			html! {
				<div class="accordion my-2" id="all-lineages">
					{lineage_order.into_iter().map(move |(source_id, lineage)| {
						let is_current_selection = is_lineage_in(&lineage.name, &[*selected_slot]);
						let is_otherwise_selected = is_lineage_in(&lineage.name, &other_slots[..]);

						let on_select = on_lineage_selected.reform({
							let target_slot = *selected_slot;
							let source_id = source_id.clone();
							move |_| (target_slot, Some(source_id.clone()))
						});
						html! {
							<LineageItem
								parent_collapse={"#all-lineages"}
								name={lineage.name.clone()}
								current_slot_name={current_slot_name.cloned()}
								{is_current_selection}
								{is_otherwise_selected}
								can_select_again={lineage.can_select_twice}
								{on_select}
							>
								<div style="white-space: pre-line;">
									{lineage.description.clone()}
								</div>
							</LineageItem>
						}
					}).collect::<Vec<_>>()}
				</div>
			}
		}
		SelectedSlot::EditContents(slot_idx) => html! {{format!("{slot_idx}")}},
	};

	html! {<>
		<div class="form-check form-switch m-2">
			<label for="useLineages" class="form-check-label">{"Use Lineages & Upbringings"}</label>
			<input  id="useLineages" class="form-check-input"
				type="checkbox" role="switch"
				checked={*use_lineages}
				onchange={toggle_lineages}
			/>
			<div id="useLineagesHelpBlock" class="form-text">{HELP_TEXT}</div>
		</div>
		<div class="row">
			<div class="col">
				<h4>{"Lineages"}</h4>
				<p>{"Select lineages and edit feature selections here."}</p>
				{lineage_slots}
				{panel_content}
			</div>
			<div class="col">
				<h4>{"Upbringings"}</h4>
			</div>
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct LineageItemProps {
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
fn LineageItem(
	LineageItemProps {
		parent_collapse,
		name,
		current_slot_name,
		is_current_selection,
		is_otherwise_selected,
		can_select_again,
		children,
		on_select,
	}: &LineageItemProps,
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
