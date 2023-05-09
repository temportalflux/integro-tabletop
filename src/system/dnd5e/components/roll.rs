use super::GeneralProp;
use crate::system::dnd5e::data::roll;
use yew::prelude::*;

#[function_component]
pub fn ModifierIcon(props: &GeneralProp<roll::Modifier>) -> Html {
	let mut classes = classes!("icon");
	classes.push(match &props.value {
		roll::Modifier::Advantage => "advantage",
		roll::Modifier::Disadvantage => "disadvantage",
	});
	html! { <span class={classes} /> }
}
