use crate::components::use_media_query;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct MobileProps {
	pub threshold: usize,
	#[prop_or_default]
	pub children: Children,
}

#[derive(Clone, PartialEq)]
struct MobileState(UseStateHandle<bool>);
impl MobileState {
	fn is_larger_than_mobile(&self) -> bool {
		*self.0
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
	Desktop,
	Mobile,
}

#[function_component]
pub fn Provider(MobileProps { threshold, children }: &MobileProps) -> Html {
	let is_larger_than_mobile = use_media_query(&format!("(min-width: {threshold}px)"));
	html! {
		<ContextProvider<MobileState> context={MobileState(is_larger_than_mobile)}>
			{children.clone()}
		</ContextProvider<MobileState>>
	}
}

#[hook]
pub fn use_mobile_kind() -> Kind {
	let state = use_context::<MobileState>().unwrap();
	match state.is_larger_than_mobile() {
		true => Kind::Desktop,
		false => Kind::Mobile,
	}
}
