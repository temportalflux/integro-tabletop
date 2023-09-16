use yew::html::ChildrenProps;
pub use yew::prelude::*;

use crate::components::stop_propagation;

#[derive(Clone, PartialEq)]
pub struct Control {

}

#[function_component]
pub fn Provider(props: &ChildrenProps) -> Html {
	let control = Control {};
	html! {
		<ContextProvider<Control> context={control.clone()}>
			{props.children.clone()}
		</ContextProvider<Control>>
	}
}

#[function_component]
pub fn ContextMenu() -> Html {
	let control = use_context::<Control>().unwrap();
	let is_active = use_state_eq(|| false);

	let toggle_activity = Callback::from({
		let is_active = is_active.clone();
		move |evt: web_sys::MouseEvent| {
			evt.stop_propagation();
			is_active.set(!*is_active);
		}
	});

	let mut root_classes = classes!("context-menu");
	if *is_active {
		root_classes.push("active");
	}

	let tab_content = match *is_active {
		false => html!(<>
			<i class="bi me-1 bi-chevron-double-up" />
			{"Expand"}
		</>),
		true => html!(<>
			<i class="bi me-1 bi-chevron-double-down" />
			{"Collapse"}
		</>),
	};

	html! {
		<div class={root_classes}>
			<div class="backdrop" onclick={toggle_activity.clone()} />
			<div class="panel">
				<div class="spacer" />
				<div class="content-box mx-3">
					<div class="tab-origin">
						<div class="tab px-2" onclick={toggle_activity.clone()}>
							{tab_content}
						</div>
					</div>
					<div class="card">
						<div class="card-body">
						</div>
					</div>
				</div>
			</div>
		</div>
	}
}
