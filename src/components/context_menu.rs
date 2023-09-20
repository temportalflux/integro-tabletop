use std::collections::VecDeque;
use std::rc::Rc;
use yew::html::ChildrenProps;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Control(UseReducerHandle<State>);
impl From<UseReducerHandle<State>> for Control {
	fn from(value: UseReducerHandle<State>) -> Self {
		Self(value)
	}
}
impl std::ops::Deref for Control {
	type Target = State;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, PartialEq, Default)]
pub struct State {
	is_shown: bool,
	stack: VecDeque<Item>,
}

#[derive(Clone, PartialEq)]
pub struct Item {
	pub display_name: AttrValue,
	pub html: Html,
}
impl Item {
	pub fn new(display_name: impl Into<AttrValue>, html: impl Into<Html>) -> Self {
		Self {
			display_name: display_name.into(),
			html: html.into(),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum Action {
	Expand,
	Collapse,

	OpenRoot(Item),
	OpenSubpage(Item),
	CloseCurrent,
}

impl Action {
	pub fn open_root(display_name: impl Into<AttrValue>, html: impl Into<Html>) -> Self {
		Self::OpenRoot(Item::new(display_name, html))
	}

	pub fn open_child(display_name: impl Into<AttrValue>, html: impl Into<Html>) -> Self {
		Self::OpenSubpage(Item::new(display_name, html))
	}
}

impl Reducible for State {
	type Action = Action;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		match action {
			Action::Expand => Rc::new(Self {
				is_shown: true,
				stack: self.stack.clone(),
			}),
			Action::Collapse => Rc::new(Self {
				is_shown: false,
				stack: self.stack.clone(),
			}),
			Action::OpenRoot(item) => Rc::new(Self {
				is_shown: true,
				stack: vec![item].into(),
			}),
			Action::OpenSubpage(item) => {
				let mut stack = self.stack.clone();
				stack.push_back(item);
				Rc::new(Self {
					is_shown: true,
					stack,
				})
			}
			Action::CloseCurrent => {
				let mut stack = self.stack.clone();
				stack.pop_back();
				Rc::new(Self {
					is_shown: !stack.is_empty(),
					stack,
				})
			}
		}
	}
}

impl Control {
	pub fn dispatch(&self, action: Action) {
		self.0.dispatch(action);
	}

	fn toggle_shown(&self) {
		self.dispatch(match self.is_shown {
			true => Action::Collapse,
			false => Action::Expand,
		});
	}

	fn toggle_shown_fn(&self) -> Callback<web_sys::MouseEvent, ()> {
		Callback::from({
			let control = self.clone();
			move |evt: web_sys::MouseEvent| {
				evt.stop_propagation();
				control.toggle_shown();
			}
		})
	}

	fn close_current(&self) {
		self.dispatch(Action::CloseCurrent);
	}

	fn close_current_fn<FnIn>(&self) -> Callback<FnIn, ()> {
		Callback::from({
			let control = self.clone();
			move |_: FnIn| control.close_current()
		})
	}
}

#[function_component]
pub fn Provider(props: &ChildrenProps) -> Html {
	let control = Control::from(use_reducer(|| State::default()));
	html! {
		<ContextProvider<Control> context={control.clone()}>
			{props.children.clone()}
		</ContextProvider<Control>>
	}
}

#[hook]
pub fn use_control_action<F, FnIn>(callback: F) -> Callback<FnIn, ()>
where
	F: Fn(FnIn) -> Action + 'static,
{
	let control = use_context::<Control>().unwrap();
	Callback::from(move |arg: FnIn| {
		control.0.dispatch(callback(arg));
	})
}

#[hook]
pub fn use_close_fn<FnIn>() -> Callback<FnIn, ()> {
	let control = use_context::<Control>().unwrap();
	control.close_current_fn()
}

#[derive(Clone, PartialEq)]
pub struct ActiveContext;

#[function_component]
pub fn ContextMenu() -> Html {
	let control = use_context::<Control>().unwrap();

	let mut root_classes = classes!("context-menu");
	if control.is_shown {
		root_classes.push("active");
	}

	html! {
		<div class={root_classes}>
			<div class="backdrop" onclick={control.toggle_shown_fn()} />
			<div class="panel">
				<div class="spacer" />
				<div class="content-box mx-3">
					<div class="tab-origin">
						<TabButton />
					</div>
					<div class="card">
						<div class="card-header">
							<Breadcrumb />
							<BackButton />
						</div>
						<div class="card-body">
							<ContextProvider<ActiveContext> context={ActiveContext}>
								{match control.stack.back() {
									None => html!(),
									Some(item) => item.html.clone(),
								}}
							</ContextProvider<ActiveContext>>
						</div>
					</div>
				</div>
			</div>
		</div>
	}
}

#[function_component]
fn TabButton() -> Html {
	let control = use_context::<Control>().unwrap();

	let mut classes = classes!("tab", "px-2");
	if !control.is_shown && control.stack.is_empty() {
		classes.push("disabled");
	}

	html! {
		<div class={classes} onclick={control.toggle_shown_fn()}>
			{match (control.is_shown, control.stack.is_empty()) {
				(false, false) => html!(<>
					<i class="bi me-1 bi-chevron-double-up" />
					{"Expand"}
				</>),
				(false, true) => html!("No Context"),
				(true, _) => html!(<>
					<i class="bi me-1 bi-chevron-double-down" />
					{"Collapse"}
				</>),
			}}
		</div>
	}
}

#[function_component]
fn BackButton() -> Html {
	let control = use_context::<Control>().unwrap();
	let onclick = control
		.close_current_fn()
		.reform(|evt: web_sys::MouseEvent| {
			evt.stop_propagation();
		});
	html! {
		<button type="button" class="btn close ms-auto" {onclick}>
			{match control.stack.len() {
				1 => html! {<>
					<i class="bi bi-x-lg me-1" />
					{"Close"}
				</>},
				_ => html! {<>
					<i class="bi bi-chevron-left me-1" />
					{"Back"}
				</>},
			}}
		</button>
	}
}

#[function_component]
fn Breadcrumb() -> Html {
	let control = use_context::<Control>().unwrap();
	html! {
		<nav>
			<ol class="breadcrumb">
				{control.stack.iter().enumerate().map(|(idx, item)| {
					let mut classes = classes!("breadcrumb-item");
					if idx + 1 == control.stack.len() {
						classes.push("active");
					}
					html!(<li class={classes}>{&item.display_name}</li>)
				}).collect::<Vec<_>>()}
			</ol>
		</nav>
	}
}
