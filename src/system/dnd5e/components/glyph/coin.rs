use crate::system::dnd5e::{components::glyph::Glyph, data::currency};
use yew::prelude::*;

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
pub fn Coin(CoinProps { kind, tag, classes, large }: &CoinProps) -> Html {
	let mut classes = classes!("currency", classes.clone());
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
	html!(<Glyph tag={tag.clone()} {classes} />)
}
