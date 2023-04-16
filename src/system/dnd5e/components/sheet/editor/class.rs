use crate::system::dnd5e::{
	components::{
		editor::{feature, mutator_list},
		SharedCharacter,
	},
	data::{
		character::{ActionEffect, Persistent},
		Class, Level,
	},
	DnD5e,
};
use convert_case::{Case, Casing};
use itertools::Itertools;
use std::collections::HashSet;
use yew::prelude::*;

#[function_component]
pub fn ClassTab() -> Html {
	html! {<div class="mx-4 mt-3">
		<ActiveClassList />
		<BrowserSection />
	</div>}
}

#[function_component]
fn BrowserSection() -> Html {
	let browser_collapse = use_node_ref();
	let is_browser_open = use_state_eq(|| false);
	let toggle_browser = Callback::from({
		let is_browser_open = is_browser_open.clone();
		move |_| {
			is_browser_open.set(!*is_browser_open);
		}
	});
	html! {<>
		<div class="d-flex justify-content-center">
			<ClassBrowerToggle is_open={*is_browser_open} on_click={toggle_browser.clone()} />
		</div>
		<div class="collapse" id="classBrowser" ref={browser_collapse}>
			<ClassBrowser on_added={toggle_browser.clone()} />
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct ClassBrowerToggleProps {
	is_open: bool,
	on_click: Callback<()>,
}

#[function_component]
fn ClassBrowerToggle(
	ClassBrowerToggleProps { is_open, on_click }: &ClassBrowerToggleProps,
) -> Html {
	let mut classes = classes!("btn");
	classes.push(match *is_open {
		false => "btn-outline-success",
		true => "btn-danger",
	});
	let text = match *is_open {
		true => "Close Class Browser",
		false => "Open Class Browser",
	};
	html! {
		<button
			type="button" class={classes}
			data-bs-toggle="collapse" data-bs-target="#classBrowser"
			onclick={on_click.reform(|_| ())}
		>
			{text}
		</button>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ClassBrowserProps {
	on_added: Callback<()>,
}

#[function_component]
fn ClassBrowser(ClassBrowserProps { on_added }: &ClassBrowserProps) -> Html {
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let state = use_context::<SharedCharacter>().unwrap();
	let update = use_force_update();
	let added_classes = state
		.persistent()
		.classes
		.iter()
		.map(|class| class.source_id.clone())
		.collect::<HashSet<_>>();
	let class_iter = system
		.classes
		.iter()
		.filter(|(id, _)| !added_classes.contains(*id))
		.sorted_by(|(_, a), (_, b)| a.name.cmp(&b.name));
	html! {
		<div class="accordion my-2" id="all-entries">
			{class_iter.map(|(source_id, class)| {
				let id = class.name.to_case(Case::Snake);
				html! {
					<div class="accordion-item">
						<h2 class="accordion-header">
							<button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target={format!("#{id}")}>
								{class.name.clone()}
							</button>
						</h2>
						<div {id} class="accordion-collapse collapse" data-bs-parent={"#all-entries"}>
							<div class="accordion-body">
								<button
									type="button" class="btn btn-success my-1 w-100"
									data-bs-toggle="collapse" data-bs-target="#classBrowser"
									onclick={Callback::from({
										let system = system.clone();
										let source_id = source_id.clone();
										let state = state.clone();
										let on_added = on_added.clone();
										let update = update.clone();
										move |_| {
											let Some(class_to_add) = system.classes.get(&source_id).cloned() else { return; };
											state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
												persistent.add_class(class_to_add);
												Some(ActionEffect::Recompile)
											}));
											on_added.emit(());
											update.force_update();
										}
									})}
								>{"Add"}</button>
								{class_body(class, false)}
							</div>
						</div>
					</div>
				}
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[function_component]
fn ActiveClassList() -> Html {
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let state = use_context::<SharedCharacter>().unwrap();
	let onclick_add = Callback::from({
		let system = system.clone();
		let state = state.clone();
		move |idx: usize| {
			let system = system.clone();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				let Some(dst) = persistent.classes.get_mut(idx) else { return None; };
				let Some(src) = system.classes.get(&dst.source_id) else { return None; };
				let next_level_idx = dst.levels.len();
				let Some(src_level) = src.levels.get(next_level_idx) else { return None; };
				dst.levels.push(src_level.clone());
				Some(ActionEffect::Recompile)
			}));
		}
	});
	let remove_class = Callback::from({
		let state = state.clone();
		move |idx| {
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				let _ = persistent.classes.remove(idx);
				Some(ActionEffect::Recompile)
			}));
		}
	});
	let onclick_remove = Callback::from({
		let state = state.clone();
		move |idx: usize| {
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				let remove_class = {
					let Some(class) = persistent.classes.get_mut(idx) else { return None; };
					let _ = class.levels.pop();
					class.levels.is_empty()
				};
				if remove_class {
					let _ = persistent.classes.remove(idx);
				}
				Some(ActionEffect::Recompile)
			}));
		}
	});
	html! {<>
		{state.persistent().classes.iter().enumerate().map(|(idx, class)| {
			html! {
				<div class="card my-2">
					<div class="card-header d-flex">
						{class.name.clone()}
						<button
							type="button"
							class="btn-close ms-auto" aria-label="Close"
							onclick={remove_class.reform(move |_| idx)}
						/>
					</div>
					<div class="card-body">
						{class_body(class, true)}
						<div class="d-flex justify-content-center">
							<button
								type="button" class="btn btn-success mx-2"
								onclick={onclick_add.reform(move |_| idx)}
							>{"Add Level"}</button>
							<button
								type="button" class="btn btn-danger mx-2"
								onclick={onclick_remove.reform(move |_| idx)}
							>{match class.levels.len() {
								1 => "Remove Class".to_owned(),
								_ => format!("Remove Level {}", class.levels.len()),
							}}</button>
						</div>
					</div>
				</div>
			}
		}).collect::<Vec<_>>()}
	</>}
}

fn class_body(value: &Class, show_selectors: bool) -> Html {
	let class_level_div_id = format!("{}-level", value.name.to_case(Case::Snake));
	let hit_die = value.hit_die;
	html! {<>
		<div class="text-block">
			{value.description.clone()}
		</div>
		<span>
			{"Hit Die: "}
			{hit_die.to_string()}
		</span>
		{mutator_list(&value.mutators, show_selectors)}

		<div class="my-2">
			{value.levels.iter().enumerate()
			.filter(|(_, level)| show_selectors || !level.is_empty())
			.map(|(idx, level)| {
				html! {
					<CollapsableCard
						id={format!("{}{}", class_level_div_id, idx)}
						collapse_btn_classes={level.is_empty().then_some("v-hidden").unwrap_or_default()}
						header_content={{
							html! {<>
								<span>{"Level "}{idx + 1}</span>
								{show_selectors.then(move || html! {
									<span class="ms-auto">
										{"Hit Points: "}
										{"TODO"}
										{" / "}
										{hit_die.value()}
									</span>
								}).unwrap_or_default()}
							</>}
						}}
					>
						{level_body(level, show_selectors)}
					</CollapsableCard>
				}
			}).collect::<Vec<_>>()}
		</div>

	</>}
}

#[derive(Clone, PartialEq, Properties)]
pub struct CollapsableCardProps {
	pub id: AttrValue,

	#[prop_or_default]
	pub root_classes: Classes,

	#[prop_or_default]
	pub header_classes: Classes,
	#[prop_or_default]
	pub header_content: Html,
	#[prop_or_default]
	pub collapse_btn_classes: Classes,

	#[prop_or_default]
	pub body_classes: Classes,

	#[prop_or_default]
	pub children: Children,
}
#[function_component]
pub fn CollapsableCard(props: &CollapsableCardProps) -> Html {
	let CollapsableCardProps {
		id,
		root_classes,
		header_classes,
		header_content,
		collapse_btn_classes,
		body_classes,
		children,
	} = props;
	static START_SHOWN: bool = false;
	let card_classes = classes!("card", "collapsable", root_classes.clone());
	let header_classes = classes!(
		"card-header",
		"d-flex",
		"align-items-center",
		header_classes.clone()
	);
	let body_classes = classes!("card-body", body_classes.clone());
	let mut collapse_btn_classes = classes!("arrow", "me-2", collapse_btn_classes.clone());
	let mut collapse_div_classes = classes!("collapse");
	match START_SHOWN {
		true => {
			collapse_div_classes.push("show");
		}
		false => {
			collapse_btn_classes.push("collapsed");
		}
	}

	html! {
		<div class={card_classes}>
			<div class={header_classes}>
				<button
					role="button" class={collapse_btn_classes}
					data-bs-toggle="collapse"
					data-bs-target={format!("#{}", id.as_str())}
				/>
				{header_content.clone()}
			</div>
			<div {id} class={collapse_div_classes}>
				<div class={body_classes}>
					{children.clone()}
				</div>
			</div>
		</div>
	}
}

fn level_body(value: &Level, show_selectors: bool) -> Html {
	html! {<>
		{mutator_list(&value.mutators, show_selectors)}
		{value.features.iter().map(|f| feature(f, show_selectors)).collect::<Vec<_>>()}
	</>}
}
