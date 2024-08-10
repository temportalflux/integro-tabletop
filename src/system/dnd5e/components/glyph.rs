use crate::{
	components::Style,
	system::dnd5e::{
		components::GeneralProp,
		data::{self, roll},
	},
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
	pub id: Option<AttrValue>,
	#[prop_or_default]
	pub classes: Classes,
	#[prop_or_default]
	pub style: Style,
	#[prop_or_default]
	pub aria_label: Option<AttrValue>,
}

#[function_component]
pub fn Glyph(GlyphProps { tag, id, classes, style, aria_label }: &GlyphProps) -> Html {
	html!(<@{tag.as_str().to_owned()} {id}
		class={classes!("glyph", classes.clone())}
		style={style.clone()}
		aria-label={aria_label}
	/>)
}

#[function_component]
pub fn Ability(GeneralProp::<data::Ability> { value }: &GeneralProp<data::Ability>) -> Html {
	html!(<Glyph tag="i" classes={classes!("ability", value.long_name().to_lowercase())} />)
}

#[derive(Clone, PartialEq, Properties)]
pub struct RollModifierProps {
	pub value: roll::Modifier,
	#[prop_or_default]
	pub classes: Classes,
	#[prop_or_default]
	pub style: Style,
}

#[function_component]
pub fn RollModifier(props: &RollModifierProps) -> Html {
	let classes = classes!(props.classes.clone(), match &props.value {
		roll::Modifier::Advantage => "advantage",
		roll::Modifier::Disadvantage => "disadvantage",
	});
	html!(<Glyph tag="span" {classes} style={props.style.clone()} aria_label={format!("{:?}", props.value)} />)
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

#[function_component]
pub fn DamageTypeGlyph(props: &GeneralProp<data::DamageType>) -> Html {
	html!(<Glyph tag="span" classes={classes!(
		"damage_type",
		props.value.to_string().to_lowercase(),
	)} />)
}
