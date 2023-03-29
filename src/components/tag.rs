use yew::{html::ChildrenProps, prelude::*};

#[derive(Clone, PartialEq, Properties)]
pub struct TagProps {
	#[prop_or_default]
	pub active: bool,
	#[prop_or_default]
	pub classes: Classes,
	#[prop_or_default]
	pub children: Children,
	/// Emitted when the span is clicked, with the argument
	/// indicating if tag is currently active or not.
	#[prop_or_default]
	pub on_click: Option<Callback<bool>>,
}

#[function_component]
pub fn Tag(
	TagProps {
		active,
		classes,
		children,
		on_click,
	}: &TagProps,
) -> Html {
	let mut classes = classes!("tag", classes.clone());
	if *active {
		classes.push("active");
	}
	let is_active = *active;
	let onclick = on_click
		.as_ref()
		.map(|callback| callback.reform(move |_: MouseEvent| is_active));
	html! {
		<span class={classes} {onclick}>{children.clone()}</span>
	}
}

#[function_component]
pub fn Tags(ChildrenProps { children }: &ChildrenProps) -> Html {
	html! {
		<div class="tags">{children.clone()}</div>
	}
}
