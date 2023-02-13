use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Placement {
	Top,
	Bottom,
	Left,
	Right,
}

impl Placement {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Top => "top",
			Self::Bottom => "bottom",
			Self::Left => "left",
			Self::Right => "right",
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
	#[prop_or_else(|| "div".into())]
	pub tag: String,
	#[prop_or_default]
	pub classes: Classes,

	#[prop_or(Placement::Bottom)]
	pub placement: Placement,
	#[prop_or_default]
	pub use_html: bool,

	#[prop_or_default]
	pub content: Option<String>,

	#[prop_or_default]
	pub children: Children,
}

#[function_component]
pub fn Component(
	Props {
		tag,
		classes,
		placement,
		content,
		use_html,
		children,
	}: &Props,
) -> Html {
	html! {<@{tag.clone()}
		class={classes.clone()}
		data-bs-toggle={content.is_some().then(|| "tooltip").unwrap_or("")}
		data-bs-placement={placement.as_str()}
		data-bs-html={format!("{use_html}")}
		data-bs-title={content.clone()}
	>
		{children.clone()}
	</@>}
}
