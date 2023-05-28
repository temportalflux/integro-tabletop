use std::rc::Rc;
use yew::prelude::*;

mod dispatch;
pub use dispatch::*;
mod progress;
pub use progress::*;
mod list;
pub use list::*;

#[derive(Clone, PartialEq)]
pub struct View(UseReducerHandle<List>);
impl View {
	pub fn iter(&self) -> impl Iterator<Item = &Handle> + '_ {
		let iter_ids = self.0.display_order.iter();
		let iter_handles = iter_ids.filter_map(|id| self.0.tasks.get(id));
		iter_handles
	}
}

#[function_component]
pub fn Provider(props: &html::ChildrenProps) -> Html {
	let list = use_reducer_eq(|| List::default());
	let dispatch = Dispatch(Rc::new(list.clone()));
	let view = View(list);
	html! {
		<ContextProvider<Dispatch> context={dispatch}>
			<ContextProvider<View> context={view}>
				{props.children.clone()}
			</ContextProvider<View>>
		</ContextProvider<Dispatch>>
	}
}
