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
