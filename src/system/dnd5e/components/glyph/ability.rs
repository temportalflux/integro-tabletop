use crate::system::dnd5e::{data, components::GeneralProp};
use yew::prelude::*;

#[function_component]
pub fn Ability(GeneralProp::<data::Ability> { value }: &GeneralProp<data::Ability>) -> Html {
	html!(<i class={classes!("glyph", "ability", value.long_name().to_lowercase())} />)
}
