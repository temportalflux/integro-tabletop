use std::rc::Rc;

use yew::{prelude::*, virtual_dom::VNode};

#[derive(Clone, Copy, PartialEq)]
pub enum NavDisplay {
	/// https://getbootstrap.com/docs/5.3/components/navs-tabs/#tabs
	Tabs,
	/// https://getbootstrap.com/docs/5.3/components/navs-tabs/#pills
	Pills,
}
impl Into<Classes> for NavDisplay {
	fn into(self) -> Classes {
		match self {
			Self::Tabs => classes!("nav-tabs"),
			Self::Pills => classes!("nav-pills"),
		}
	}
}

/// https://getbootstrap.com/docs/5.3/components/navs-tabs/#horizontal-alignment
#[derive(Clone, Copy, PartialEq)]
pub enum Justify {
	Start,
	Center,
	End,
}
impl Default for Justify {
	fn default() -> Self {
		Self::Start
	}
}
impl Into<Classes> for Justify {
	fn into(self) -> Classes {
		match self {
			Self::Start => classes!("justify-content-start"),
			Self::Center => classes!("justify-content-center"),
			Self::End => classes!("justify-content-end"),
		}
	}
}

/// https://getbootstrap.com/docs/5.3/components/navs-tabs/#fill-and-justify
#[derive(Clone, Copy, PartialEq)]
pub enum NavWidth {
	Fill,
	Justify,
}
impl Into<Classes> for NavWidth {
	fn into(self) -> Classes {
		match self {
			Self::Fill => classes!("nav-fill"),
			Self::Justify => classes!("nav-justified"),
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct NavProps {
	/// Classes to add to the root div containing the nav ul and the tab content div.
	#[prop_or_default]
	pub root_classes: Classes,
	/// Classes to add to the ul.nav element.
	#[prop_or_default]
	pub nav_classes: Classes,
	/// What kind of display format to use (none for just text, otherwise tabs or pills).
	#[prop_or_default]
	pub disp: Option<NavDisplay>,
	/// True if the nav should be displayed vertically in a column.
	#[prop_or_default]
	pub column: bool,
	/// How to horizontally justify the nav content.
	/// https://getbootstrap.com/docs/5.3/components/navs-tabs/#horizontal-alignment
	#[prop_or_default]
	pub justify: Justify,
	/// How the nav takes up horizontal width.
	/// https://getbootstrap.com/docs/5.3/components/navs-tabs/#fill-and-justify
	#[prop_or_default]
	pub width: Option<NavWidth>,
	/// The id in `TabContent` of the default tab.
	#[prop_or_default]
	pub default_tab_id: String,
	#[prop_or_default]
	pub children: ChildrenWithProps<TabContent>,
}

#[function_component]
pub fn Nav(
	NavProps {
		root_classes,
		nav_classes,
		disp,
		column,
		justify,
		width,
		default_tab_id,
		children,
	}: &NavProps,
) -> Html {
	let default_tab_id = default_tab_id.clone();
	let selected_tab = use_state(move || default_tab_id);

	let nav_classes = {
		let mut classes = classes!("nav", nav_classes.clone());
		if let Some(disp) = disp {
			classes.push(*disp);
		}
		classes.push(*justify);
		classes.push(match *column {
			false => "flex-row",
			true => "flex-column",
		});
		if let Some(width) = width {
			classes.push(*width);
		}
		classes
	};

	let mut nav_items = Vec::with_capacity(children.len());
	let mut tab_children = Vec::with_capacity(children.len());
	for mut child in children.iter() {
		let mut props = Rc::make_mut(&mut child.props);
		props.active = *selected_tab == props.id;

		let mut classes = classes!("nav-link");
		if props.active {
			classes.push("active");
		}
		let onclick = {
			let id = props.id.clone();
			let selected_tab = selected_tab.clone();
			Callback::from(move |_| selected_tab.set(id.clone()))
		};
		nav_items.push(html! {
			<li class="nav-item" role="presentation">
				<button class={classes} type="button" role="tab" {onclick}>{props.title.clone()}</button>
			</li>
		});
		tab_children.push(html! { <TabContent ..props.clone() /> });
	}

	html! {
		<div class={root_classes.clone()}>
			<ul class={nav_classes} role="tablist">
				{nav_items}
			</ul>
			<div class="tab-content">
				{tab_children}
			</div>
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct TabContentProps {
	pub id: String,
	pub title: VNode,
	#[prop_or_default]
	pub children: Children,

	// INTERNAL ONLY: Provided by `Nav`.
	#[prop_or_default]
	pub active: bool,
}

#[function_component]
pub fn TabContent(
	TabContentProps {
		id,
		title: _,
		children,
		active,
	}: &TabContentProps,
) -> Html {
	let mut classes = classes!("tab-pane");
	if *active {
		classes.push("active");
	}
	html! {
		<div class={classes} role="tabpanel" id={id.clone()}>
			{children.clone()}
		</div>
	}
}
