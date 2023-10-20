use crate::components::Style;
use any_range::AnyRange;
use enumset::EnumSetType;
use std::{collections::BTreeSet, ops::RangeInclusive};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct TickedProps {
	#[prop_or_default]
	pub classes: Classes,
	#[prop_or_default]
	pub style: Style,
	#[prop_or_default]
	pub show_labels: bool,

	#[prop_or(TickDisplay::AllTicks)]
	pub ticked_bar_range: TickDisplay,
	#[prop_or(Some(TickDisplay::BoundsOnly))]
	pub ticked_value_range: Option<TickDisplay>,

	pub bar_range: RangeInclusive<i64>,
	pub value_range: AnyRange<i64>,
}

#[derive(EnumSetType)]
pub enum TickDisplay {
	// Show all ticks in the range.
	AllTicks,
	// Show ticks only for the min and max of the range.
	BoundsOnly,
}

#[function_component]
pub fn Ticked(props: &TickedProps) -> Html {
	let TickedProps {
		classes,
		style,
		show_labels,
		ticked_bar_range,
		ticked_value_range,
		bar_range,
		value_range,
	} = props;
	let bar_min = *bar_range.start();
	let bar_max = *bar_range.end();
	let tick_count = bar_max - bar_min;

	let root_classes = classes!("progress-ticked", classes.clone());
	let style = Style::from([("--tick-count", tick_count.to_string())]) + style;

	let mut shown_ticks: BTreeSet<i64> = match ticked_bar_range {
		TickDisplay::AllTicks => bar_range.clone().into_iter().collect(),
		TickDisplay::BoundsOnly => [*bar_range.start(), *bar_range.end()].into(),
	};
	let value_min = match value_range.start_bound() {
		std::ops::Bound::Included(v) => *v,
		std::ops::Bound::Excluded(v) => *v,
		std::ops::Bound::Unbounded => bar_min,
	};
	let value_max = match value_range.end_bound() {
		std::ops::Bound::Included(v) => *v,
		std::ops::Bound::Excluded(v) => *v,
		std::ops::Bound::Unbounded => bar_max,
	};
	if let Some(tick_range) = ticked_value_range {
		match tick_range {
			TickDisplay::AllTicks => shown_ticks.extend((value_min..=value_max).into_iter()),
			TickDisplay::BoundsOnly => shown_ticks.extend([value_min, value_max]),
		}
	}

	let ticks = shown_ticks
		.into_iter()
		.map(|i| {
			let mut class = classes!("tick");
			if value_range.contains(&i) {
				class.push("color-fill");
			}
			let style = Style::from([("--tick-position", i.to_string())]);
			let label = match show_labels {
				false => html!(),
				true => html!(<span class="label" style="--width: 20px; height: 20px;">{i}</span>),
			};
			html!(<div {class} {style}>{label}</div>)
		})
		.collect::<Vec<_>>();

	let spacer_progress = Style::from([("--tick-position", value_min)]);
	let real_progress = Style::from([("--tick-position", value_max - value_min)]);

	html! {
		<div class={root_classes} {style}>
			<div class="progress" role="progressbar">
				{ticks}
				<div class="progress-bar spacer tick-amt" style={spacer_progress} />
				<div class="progress-bar tick-amt" style={real_progress} />
			</div>
		</div>
	}
}
