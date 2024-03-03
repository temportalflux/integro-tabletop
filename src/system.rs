use std::sync::Arc;
use yew::prelude::*;

pub mod dnd5e;
mod source;
pub use source::*;

pub mod block;
pub use block::Block;
pub use block::Registry as BlockRegistry;

mod registry;
pub use registry::Registry as Depot;
pub use registry::*;

pub mod mutator;
pub use mutator::Mutator;
pub mod evaluator;
pub use evaluator::Evaluator;
pub mod generator;
pub use generator::Generator;

pub mod generics;
pub use generics::Registry as NodeRegistry;

pub mod core {
	pub use super::generics::Registry as NodeRegistry;
	pub use super::{ModuleId, SourceId, System};
}

pub trait System {
	fn id() -> &'static str
	where
		Self: Sized;
	fn get_id(&self) -> &'static str;
	fn blocks(&self) -> &block::Registry;
	fn generics(&self) -> &Arc<NodeRegistry>;
}

#[function_component]
pub fn Provider(props: &html::ChildrenProps) -> Html {
	let depot = use_state(|| {
		let mut builder = Depot::builder();
		builder.insert(dnd5e::DnD5e::new());
		builder.build()
	});
	html! {
		<ContextProvider<Depot> context={(*depot).clone()}>
			{props.children.clone()}
		</ContextProvider<Depot>>
	}
}
