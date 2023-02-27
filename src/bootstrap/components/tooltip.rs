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
	pub tag: AttrValue,
	#[prop_or_default]
	pub classes: Classes,
	#[prop_or_default]
	pub style: AttrValue,

	#[prop_or(Placement::Bottom)]
	pub placement: Placement,
	#[prop_or_default]
	pub use_html: bool,

	#[prop_or_default]
	pub content: Option<AttrValue>,

	#[prop_or_default]
	pub children: Children,
}

#[function_component]
pub fn Component(
	Props {
		tag,
		classes,
		style,
		placement,
		content,
		use_html,
		children,
	}: &Props,
) -> Html {
	let node = use_node_ref();
	use_effect_with_deps(
		move |(node, _)| {
			if let Some(node) = node.get() {
				crate::bootstrap::Tooltip::new(
					node.into(),
					wasm_bindgen::JsValue::from("{}".to_owned()),
				);
			}
		},
		(node.clone(), content.clone()),
	);

	html! {<@{tag.as_str().to_owned()} ref={node}
		class={classes.clone()}
		{style}
		data-bs-toggle={content.is_some().then(|| "tooltip").unwrap_or("")}
		data-bs-placement={placement.as_str()}
		data-bs-html={format!("{use_html}")}
		data-bs-title={content.clone()}
	>
		{children.clone()}
	</@>}
}
