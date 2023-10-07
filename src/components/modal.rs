use crate::bootstrap;
use std::rc::Rc;
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::{html::ChildrenProps, prelude::*};

// TODO: Use portals to solve the "modal contents require page-specific-provider" problem:
// https://yew.rs/docs/advanced-topics/portals

#[function_component]
pub fn Provider(props: &ChildrenProps) -> Html {
	let modal_dispatcher = Context::from(use_reducer(|| State::default()));
	html! {
		<ContextProvider<Context> context={modal_dispatcher.clone()}>
			{props.children.clone()}
		</ContextProvider<Context>>
	}
}

/// Dispatches reducer messages about the [GeneralPurpose] modal.
/// Created by calling `use_reducer(|| State::default()).into()` in a component (for use in a [ContextProvider]).
#[derive(Clone, PartialEq)]
pub struct Context(UseReducerHandle<State>);
impl From<UseReducerHandle<State>> for Context {
	fn from(value: UseReducerHandle<State>) -> Self {
		Self(value)
	}
}
impl Context {
	pub fn callback<T, F>(&self, fn_action: F) -> Callback<T, ()>
	where
		F: Fn(T) -> Action + 'static,
	{
		let handle = self.0.clone();
		Callback::from(move |input: T| {
			handle.dispatch(fn_action(input));
		})
	}

	pub fn dispatch(&self, action: Action) {
		self.0.dispatch(action);
	}
}
impl std::ops::Deref for Context {
	type Target = State;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// The global-context state of the [GeneralPurpose] modal.
#[derive(PartialEq, Default, Debug)]
pub struct State {
	/// If the modal should show/hide.
	/// If none, no action is taken. Otherwise, the bool determines if the
	/// [Bootstrap Modal](bootstrap::Modal) should [show](bootstrap::Modal::show)
	/// or [hide](bootstrap::Modal::hide).
	pub should_show: Option<bool>,
	/// The properties for the modal to use to control displayed Html.
	/// None if the modal should be empty (i.e. not displayed).
	pub props_stack: Vec<Props>,
	pub next_pending: Option<Props>,
}
impl State {
	/// If self contains props, calls the provided mapping function to unpack some data.
	/// Otherwise, returns the default for the desired type.
	pub fn map_props<F, T>(&self, map: F) -> T
	where
		F: FnOnce(&Props) -> T,
		T: Default,
	{
		self.props_stack.last().map(map).unwrap_or_default()
	}
}

/// The actions that can be dispatched to [State] via [Context].
#[derive(Clone, PartialEq)]
pub enum Action {
	/// Open the [GeneralPurpose] modal with some props, using the show action in bootstrap-js.
	Open(Props),
	/// Close the [GeneralPurpose] modal, using the hide action and clearing [Props] data in [State].
	Close,
	/// Should only be used by [GeneralPurpose] when the [Close] action is complete.
	Closed,
}

impl Reducible for State {
	type Action = Action;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		match action {
			Action::Open(props) => {
				if self.props_stack.is_empty() {
					Rc::new(Self {
						// send the signal to open the modal since it was previously closed (empty stack)
						should_show: Some(true),
						props_stack: vec![props],
						next_pending: None,
					})
				} else {
					// send the signal to close the modal, since it is currently open (non-empty stack).
					// the next modal-props are pending, waiting for the close to finish.
					Rc::new(Self {
						should_show: Some(false),
						props_stack: self.props_stack.clone(),
						next_pending: Some(props),
					})
				}
			}
			Action::Close => {
				Rc::new(Self {
					// send the signal to close the current modal
					should_show: Some(false),
					props_stack: self.props_stack.clone(),
					next_pending: None,
				})
			}
			Action::Closed => {
				if let Some(pending) = self.next_pending.clone() {
					let mut props_stack = self.props_stack.clone();
					props_stack.push(pending);
					Rc::new(Self {
						// re-open the modal with the new props
						should_show: Some(true),
						props_stack,
						next_pending: None,
					})
				} else {
					let props_stack = {
						let mut stack = self.props_stack.clone();
						stack.pop();
						stack
					};
					if !props_stack.is_empty() {
						// Modal was manually closed, we should re-open with the next item in the stack
						Rc::new(Self {
							should_show: Some(true),
							props_stack,
							next_pending: None,
						})
					} else {
						Rc::new(Self::default())
					}
				}
			}
		}
	}
}

/// Properties provided to [GeneralPurpose] via the global [State].
#[derive(Clone, PartialEq, Default, Properties, Debug)]
pub struct Props {
	#[prop_or_default]
	pub root_classes: Classes,
	/// Content to show in the `modal-content` div.
	pub content: Html,
	/// If the modal dialog should be scrollable.
	/// https://getbootstrap.com/docs/5.3/components/modal/#scrolling-long-content
	pub scrollable: bool,
	/// If the modal should be vertically centered.
	/// https://getbootstrap.com/docs/5.3/components/modal/#vertically-centered
	pub centered: bool,
}

/// The modal compont used to display any modal. Controlled via [State]/[Context].
#[function_component]
pub fn GeneralPurpose() -> Html {
	use wasm_bindgen::JsValue;

	// Grab the modal context, so whenever the state changes, this component is re-rendered.
	let context = use_context::<Context>().unwrap();
	// The DOM node for the modal root, used to send to bootstrap-js.
	let node = use_node_ref();
	// The bootstrap-js modal, generated when `node` is updated.
	// Used to show/hide the modal.
	let bootstrap = use_state(|| None);

	// Callback sent to bootstrap-js to be emited when the animation has finished.
	let js_on_hidden: Rc<Closure<dyn Fn()>> = use_memo(
		(),
		{
			let context = context.clone();
			move |_| {
				Closure::<dyn Fn()>::new(move || {
					// When the modal has fully closed, reset the context data so there is no props and no show signal.
					// This allows the context to be re-opened with the same content,
					// AND prevents the bootstrap-js show/hide functions from triggering right away
					// (the Close actions causes bootstrap::Modal::hide to be executed, see below).
					context.dispatch(Action::Closed);
				})
			}
		},
	);

	// Generate the bootstrap-js modal when a node is found.
	// Also subscribe to event listeners on that node, which are emited by bootstrap.
	use_effect_with(
		node.clone(),
		{
			let bootstrap = bootstrap.clone();
			let on_hidden = js_on_hidden.clone();
			move |node: &NodeRef| {
				bootstrap.set(bootstrap::Modal::from(node));
				if let Some(node) = node.get() {
					let _ = node.add_event_listener_with_callback(
						"hidden.bs.modal",
						(*on_hidden).as_ref().unchecked_ref(),
					);
				}
			}
		},
	);

	// Trigger bootstrap-js modal functions when context updates (for showing/hiding).
	use_effect_with(
		(context.clone(), bootstrap.is_some()),
		{
			let bootstrap = bootstrap.clone();
			move |(context, _has_modal): &(Context, bool)| {
				// If the node hasn't been found yet, we can't do anything.
				// Since the node is populated when the component first renders,
				// we can safely assume that it will exist for all future calls,
				// as long as context defaults to no-modal-exists.
				let Some(modal) = &*bootstrap else { return; };
				// Show or hide the modal if the flag has been set. If the flag isn't set (i.e. None),
				// then this is likely a data reset/update that shouldn't re-animate anything.
				// i.e. `js_on_hidden` was triggered by bootstrap-js to indicate the modal has closed.
				let Some(should_show) = &context.should_show else { return; };
				match should_show {
					true => {
						modal.show(JsValue::UNDEFINED);
					}
					false => {
						modal.hide();
					}
				}
			}
		},
	);

	let mut root_classes = classes!("modal", "fade");
	root_classes.extend(context.map_props(|props| props.root_classes.clone()));

	let mut dialog_classes = classes!("modal-dialog");
	if context.map_props(|props| props.centered) {
		dialog_classes.push("modal-dialog-centered");
	}
	if context.map_props(|props| props.scrollable) {
		dialog_classes.push("modal-dialog-scrollable");
	}

	html! {
		<div class={root_classes} id="generalModal" ref={node}>
			<div class={dialog_classes}>
				<div class="modal-content">
					{context.map_props(|props| props.content.clone())}
				</div>
			</div>
		</div>
	}
}
