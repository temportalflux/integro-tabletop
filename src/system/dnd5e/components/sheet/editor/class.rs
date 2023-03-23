use crate::system::dnd5e::{
	components::{
		editor::{mutator_list, selectors_in, feature},
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
fn ClassBrowerToggle(ClassBrowerToggleProps { is_open, on_click }: &ClassBrowerToggleProps) -> Html {
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
	let level_accordion_id = format!("{}-level", value.name.to_case(Case::Snake));
	html! {<>
		<div class="text-block">
			{value.description.clone()}
		</div>
		<span>
			{"Hit Die: "}
			{value.hit_die.to_string()}
		</span>
		{mutator_list(&value.mutators)}
		{show_selectors.then(|| selectors_in(&value.mutators)).unwrap_or_default()}

		<div class="accordion my-2" id={level_accordion_id.clone()}>
			{value.levels.iter().enumerate().filter_map(|(idx, level)| {
				if !show_selectors && level.is_empty() {
					return None;
				}
				let id = format!("{}{}", level_accordion_id, idx);
				let collapse_target = format!("#{id}");
				let body = match level.is_empty() {
					true => html! {},
					false => html! {
						<div {id} class="accordion-collapse collapse" data-bs-parent={format!("#{level_accordion_id}")}>
							<div class="accordion-body">
								{level_body(level, show_selectors)}
							</div>
						</div>
					},
				};
				Some(html! {
					<div class="accordion-item">
						<h2 class="accordion-header">
							<button
								class="accordion-button collapsed" type="button"
								data-bs-toggle="collapse" data-bs-target={collapse_target}
								disabled={level.is_empty()}
							>
								{"Level "}{idx + 1}
								{level.is_empty().then_some(" - Empty").unwrap_or_default()}
							</button>
						</h2>
						{body}
					</div>
				})
			}).collect::<Vec<_>>()}
		</div>
	</>}
}

fn level_body(value: &Level, show_selectors: bool) -> Html {
	html! {<>
		{mutator_list(&value.mutators)}
		{value.features.iter().map(|f| feature(f.inner(), show_selectors)).collect::<Vec<_>>()}
	</>}
}
