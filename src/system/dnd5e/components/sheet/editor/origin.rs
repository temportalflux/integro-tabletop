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

	let on_select_lineage = Callback::from({
		let system = system.clone();
		let state = state.clone();
		move |(target_idx, source_id): (usize, Option<SourceId>)| {
			let system = system.clone();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				let Some(slot) = persistent.lineages.get_mut(target_idx) else { return None; };
				let new_lineage = source_id.map(|id| system.lineages.get(&id)).flatten();
				if slot.as_ref() == new_lineage {
					return None;
				}
				*slot = new_lineage.cloned();
				Some(ActionEffect::Recompile) // TODO: only recompile when leaving the editor, not on every action
			}));
		}
	});
	let lineage_slots_for_items = state
		.persistent()
		.lineages
		.iter()
		.map(|slot| slot.as_ref().map(|lineage| lineage.name.clone().into()))
		.collect::<Vec<_>>();
	let lineage_slots = state
		.persistent()
		.lineages
		.iter()
		.enumerate()
		.map({
			let on_select = on_select_lineage.clone();
			move |(idx, slot)| {
				let onclick = on_select.reform(move |_| (idx, None));
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
						<div class="col-2 my-auto">
							<button
								role="button" class="btn btn-outline-danger btn-sm"
								{onclick}
								style={match slot.is_some() {
									true => "",
									false => "visibility: hidden;",
								}}
							>{"Remove"}</button>
						</div>
					</div>
				}
			}
		})
		.collect::<Vec<_>>();

	// TODO: Next steps
	// - collapse lineage accordion with button to show
	// - figure out how to present selected information (mainly for picking selections)

	let lineage_order = {
		let mut vec = system.lineages.iter().collect::<Vec<_>>();
		vec.sort_by(|(_, a), (_, b)| a.name.partial_cmp(&b.name).unwrap());
		vec
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
				<p>{"Select two (2) from the list below"}</p>
				{lineage_slots}
				<div class="accordion my-2" id="all-lineages">
					{lineage_order.into_iter().map(|(source_id, lineage)| html! {
						<LineageItem
							parent_collapse={"#all-lineages"}
							name={lineage.name.clone()}
							slots={lineage_slots_for_items.clone()}
							can_select_twice={lineage.can_select_twice}
							on_select={on_select_lineage.reform({
								let source_id = source_id.clone();
								move |(target_idx, select): (usize, bool)| {
									(target_idx, select.then(|| source_id.clone()))
								}
							})}
						>
							<div style="white-space: pre-line;">
								{lineage.description.clone()}
							</div>
						</LineageItem>
					}).collect::<Vec<_>>()}
				</div>
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
	slots: Vec<Option<AttrValue>>,
	can_select_twice: bool,
	children: Children,
	on_select: Callback<(usize, bool)>,
}

#[function_component]
fn LineageItem(
	LineageItemProps {
		name,
		parent_collapse,
		slots,
		can_select_twice,
		children,
		on_select,
	}: &LineageItemProps,
) -> Html {
	use convert_case::{Case, Casing};

	let select_btn = |target_slot_idx: usize, bonus_text: Html| {
		let onclick = on_select.reform(move |_| (target_slot_idx, true));
		html! {
			<button type="button" class="btn btn-outline-success my-1 w-100" {onclick}>
				{"Select"}{bonus_text}
			</button>
		}
	};
	let remove_btn = |target_slot_idx: usize, bonus_text: Html| {
		let onclick = on_select.reform(move |_| (target_slot_idx, false));
		html! {
			<button type="button" class="btn btn-outline-danger my-1 w-100" {onclick}>
				{"Remove"}{bonus_text}
			</button>
		}
	};
	let replace_btn = |target_slot_idx: usize, name: &AttrValue| {
		let onclick = on_select.reform(move |_| (target_slot_idx, true));
		html! {
			<button type="button" class="btn btn-outline-warning my-1 w-100" {onclick}>
				{"Replace "}{name.clone()}
			</button>
		}
	};
	let first_empty_slot = slots
		.iter()
		.enumerate()
		.filter_map(|(idx, slot)| slot.is_none().then_some(idx))
		.next();
	// The slot this item is selected in
	let selected_slots = slots
		.iter()
		.enumerate()
		.filter_map(|(idx, slot)| slot.as_ref().map(|item| (idx, item)))
		.filter_map(|(idx, item)| (item == name).then_some(idx))
		.collect::<Vec<_>>();

	let slot_buttons = match (&selected_slots[..], *can_select_twice) {
		// Not selected
		([], _) => match first_empty_slot {
			// there is an empty slot; show select action
			Some(slot_idx) => select_btn(slot_idx, html! {}),
			// no empty slots; show replace actions
			None => {
				let btns = slots
					.iter()
					.enumerate()
					.filter_map(|(idx, slot)| slot.as_ref().map(|item| (idx, item)))
					.map(|(idx, item)| replace_btn(idx, item))
					.collect::<Vec<_>>();
				html! {<>{btns}</>}
			}
		},
		// Already selected & can only select once; show remove action
		([slot_idx], false) => remove_btn(*slot_idx, html! {}),
		(_, false) => html! {}, // unimplemented, should never have multiple selections for a only-once item
		// Already selected & can be selected twice; show relevant action for each slot
		([selected_slot_idx], true) => {
			let other_slot = slots
				.iter()
				.enumerate()
				.filter_map(|(slot_idx, slot)| {
					(slot_idx != *selected_slot_idx).then_some((slot_idx, slot.as_ref()))
				})
				.next()
				.unwrap();
			html! {<>
				{remove_btn(*selected_slot_idx, html! {})}
				{match other_slot {
					(slot_idx, None) => select_btn(slot_idx, html! {{" Again"}}),
					(slot_idx, Some(item)) => replace_btn(slot_idx, item),
				}}
			</>}
		}
		(selected_slots, true) => {
			let btns = selected_slots
				.iter()
				.map(|idx| remove_btn(*idx, html! {<>{" slot "}{*idx}</>}))
				.collect::<Vec<_>>();
			html! {<>{btns}</>}
		}
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
