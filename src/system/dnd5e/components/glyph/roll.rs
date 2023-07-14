use crate::system::dnd5e::{components::GeneralProp, data::roll};
use yew::prelude::*;

#[function_component]
pub fn RollModifier(props: &GeneralProp<roll::Modifier>) -> Html {
	let mut classes = classes!("glyph");
	classes.push(match &props.value {
		roll::Modifier::Advantage => "advantage",
		roll::Modifier::Disadvantage => "disadvantage",
	});
	html!(<span class={classes} />)
}
