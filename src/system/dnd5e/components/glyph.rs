use crate::system::dnd5e::{
	components::GeneralProp,
	data::{self, roll},
};
use yew::prelude::*;

mod coin;
pub use coin::*;
mod proficiency;
pub use proficiency::*;

#[derive(Clone, PartialEq, Properties)]
pub struct GlyphProps {
	#[prop_or_else(|| "i".into())]
	pub tag: AttrValue,
	#[prop_or_default]
	pub classes: Classes,
}

#[function_component]
pub fn Glyph(GlyphProps { tag, classes }: &GlyphProps) -> Html {
	html!(<@{tag.as_str().to_owned()} class={classes!("glyph", classes.clone())} />)
}

#[function_component]
pub fn Ability(GeneralProp::<data::Ability> { value }: &GeneralProp<data::Ability>) -> Html {
	html!(<Glyph tag="i" classes={classes!("ability", value.long_name().to_lowercase())} />)
}

#[function_component]
pub fn RollModifier(props: &GeneralProp<roll::Modifier>) -> Html {
	let classes = classes!(match &props.value {
		roll::Modifier::Advantage => "advantage",
		roll::Modifier::Disadvantage => "disadvantage",
	});
	html!(<Glyph tag="span" {classes} />)
}

#[function_component]
pub fn Defense(props: &GeneralProp<crate::system::dnd5e::mutator::Defense>) -> Html {
	use crate::system::dnd5e::mutator::Defense;
	let mut classes = classes!("defense");
	classes.push(match props.value {
		Defense::Resistance => "resistance",
		Defense::Immunity => "immunity",
		Defense::Vulnerability => "vulnerability",
	});
	html!(<Glyph tag="span" {classes} />)
}
