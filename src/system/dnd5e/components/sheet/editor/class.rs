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
	let state = use_context::<SharedCharacter>().unwrap();
	html! {<>
		{state.persistent().classes.iter().map(|class| {
			html! {
				<div class="card my-2">
					<div class="card-header">{class.name.clone()}</div>
					<div class="card-body">
						{class_body(class, true)}
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
			{value.levels.iter().enumerate().filter(|(_, level)| !level.is_empty()).map(|(idx, level)| {
				let id = format!("{}{}", level_accordion_id, idx);
				html! {
					<div class="accordion-item">
						<h2 class="accordion-header">
							<button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target={format!("#{id}")}>
								{"Level "}{idx + 1}
							</button>
						</h2>
						<div {id} class="accordion-collapse collapse" data-bs-parent={format!("#{level_accordion_id}")}>
							<div class="accordion-body">
								{level_body(level, show_selectors)}
							</div>
						</div>
					</div>
				}
			}).collect::<Vec<_>>()}
		</div>
		{show_selectors.then(|| html! {<>
			<div class="d-flex justify-content-center">
				<button
					type="button" class="btn btn-success mx-2"
				>{"Add Level"}</button>
				<button
					type="button" class="btn btn-danger mx-2"
				>{format!("Remove Level {}", value.levels.len())}</button>
			</div>
		</>}).unwrap_or_default()}
	</>}
}

fn level_body(value: &Level, show_selectors: bool) -> Html {
	html! {<>
		{mutator_list(&value.mutators)}
		{value.features.iter().map(|f| feature(f.inner(), show_selectors)).collect::<Vec<_>>()}
	</>}
}
