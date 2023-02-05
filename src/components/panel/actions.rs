use crate::components::*;
use enumset::{EnumSet, EnumSetType};
use yew::prelude::*;

#[derive(EnumSetType)]
pub enum ActionTag {
	Attack,
	Action,
	BonusAction,
	Reaction,
	Other,
	LimitedUse,
}
impl ActionTag {
	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Attack => "Attack",
			Self::Action => "Action",
			Self::BonusAction => "Bonus Action",
			Self::Reaction => "Reaction",
			Self::Other => "Other",
			Self::LimitedUse => "Limited Use",
		}
	}
}

#[function_component]
pub fn Actions() -> Html {
	let selected_tags = use_state(|| EnumSet::<ActionTag>::all());

	let make_tag_html = {
		let selected_tags = selected_tags.clone();
		move |html: Html, tag_set: EnumSet<ActionTag>| {
			let active = *selected_tags == tag_set;
			let on_click = {
				let selected_tags = selected_tags.clone();
				Callback::from(move |_| selected_tags.set(tag_set))
			};
			html! { <Tag {active} {on_click}>{html}</Tag> }
		}
	};
	let mut tag_htmls = vec![make_tag_html(html! {{"All"}}, EnumSet::all())];
	for tag in EnumSet::<ActionTag>::all() {
		tag_htmls.push(make_tag_html(
			html! {{tag.display_name()}},
			EnumSet::from(tag),
		));
	}
	html! {<>
		<Tags>{tag_htmls}</Tags>
		<div style="overflow-y: scroll;">
		</div>
	</>}
}
