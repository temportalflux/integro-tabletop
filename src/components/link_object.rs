use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ObjectLinkProps {
	pub title: AttrValue,
	#[prop_or_default]
	pub subtitle: Option<AttrValue>,
	#[prop_or_default]
	pub onclick: Option<Callback<web_sys::MouseEvent>>,
	#[prop_or_default]
	pub disabled: bool,
}

#[function_component]
pub fn ObjectLink(
	ObjectLinkProps {
		title,
		subtitle,
		onclick,
		disabled,
	}: &ObjectLinkProps,
) -> Html {
	let mut classes = classes!("card", "object-link");
	if *disabled {
		classes.push("disabled");
	}

	html! {
		<div class={classes} onclick={onclick.clone()}>
			<div class="card-body">
				<div class="header">
					<p class="title">{title}</p>
					{subtitle.as_ref().map(|subtitle| html! {
						<p class="subtitle text-body-secondary">{subtitle}</p>
					})}
				</div>
				<button class="btn btn-theme btn-xs">
					<i class="bi bi-chevron-right" />
				</button>
			</div>
		</div>
	}
}
