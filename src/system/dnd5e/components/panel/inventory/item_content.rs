use yew::prelude::*;

use crate::system::dnd5e::data::item::Item;

pub fn item_body(item: &Item) -> Html {
	let desc = item.description.as_ref().map(|desc| html! { <div class="text-block">{desc.clone()}</div> }).unwrap_or_default();
	html! {<>
		{desc}
	</>}
}
