use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct AnnotatedNumberProps {
	pub value: i32,
	#[prop_or_default]
	pub show_sign: bool,
	#[prop_or_default]
	pub suffix: Option<String>,
}

#[function_component]
pub fn AnnotatedNumber(
	AnnotatedNumberProps {
		value,
		show_sign,
		suffix,
	}: &AnnotatedNumberProps,
) -> Html {
	let mut num_span_classes = classes!();
	let prefix = match show_sign {
		true => {
			num_span_classes.push("with-prefix");
			html! {
				<span class="label prefix">{match *value >= 0 {
					true => "+",
					false => "-",
				}}</span>
			}
		}
		false => html! {},
	};
	let suffix = match suffix {
		Some(suffix) => html! { <span class="label suffix">{suffix.clone()}</span> },
		None => html! {},
	};
	html! {
		<div class="annotated-number">
			<span class={num_span_classes}>
				{prefix}
				<span class="number">{value.abs()}</span>
			</span>
			{suffix}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct AnnotatedNumberCardProps {
	pub header: String,
	pub footer: String,
	pub children: ChildrenWithProps<AnnotatedNumber>,
}

#[function_component]
pub fn AnnotatedNumberCard(
	AnnotatedNumberCardProps {
		header,
		footer,
		children,
	}: &AnnotatedNumberCardProps,
) -> Html {
	html! {
		<div class="card m-2">
			<div class="card-body text-center" style="padding: 5px 5px;">
				<h6 class="card-title" style="font-size: 0.8rem;">{header.clone()}</h6>
				<div style="font-size: 26px; font-weight: 500; margin: -8px 0;">
					{children.clone()}
				</div>
				<h6 class="card-title" style="font-size: 0.8rem; margin-bottom: 0;">{footer.clone()}</h6>
			</div>
		</div>
	}
}
