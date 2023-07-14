use yew::prelude::*;
use crate::system::dnd5e::data::currency;

#[derive(Clone, PartialEq, Properties)]
pub struct CoinProps {
	pub kind: currency::Kind,
	#[prop_or_else(|| "span".into())]
	pub tag: AttrValue,
	#[prop_or_default]
	pub classes: Classes,
	#[prop_or_default]
	pub large: bool,
}

#[function_component]
pub fn Coin(
	CoinProps {
		kind,
		tag,
		classes,
		large,
	}: &CoinProps,
) -> Html {
	let mut classes = classes!("glyph", "currency", classes.clone());
	if *large {
		classes.push("lg");
	}
	classes.push(match kind {
		currency::Kind::Copper => "copper",
		currency::Kind::Silver => "silver",
		currency::Kind::Electrum => "electrum",
		currency::Kind::Gold => "gold",
		currency::Kind::Platinum => "platinum",
	});
	html! { <@{tag.as_str().to_owned()} class={classes} /> }
}
