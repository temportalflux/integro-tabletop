use crate::{
	page::characters::sheet::ViewProps,
	system::dnd5e::components::GeneralProp,
	utility::{InputExt, NotInList},
};
use enumset::{EnumSet, EnumSetType};
use std::str::FromStr;
use yew::prelude::*;

/*
Mobile-First Pages:
- Header
  - Name, Pronouns
  - Take a Rest btns
  - HP Small
  - Inspiration
- Actions & Features
- Spells
- Inventory
- Description
*/

pub mod abilities;
pub mod attributes;

#[derive(EnumSetType, Default)]
enum Page {
	#[default]
	Abilities,
	Attributes,
	Features,
	Spells,
	Inventory,
	Description,
}
impl Page {
	fn display_name(&self) -> &'static str {
		match self {
			Self::Abilities => "Abilities & Skills",
			Self::Attributes => "Attributes & Proficiencies",
			Self::Features => "Actions & Features",
			Self::Spells => "Spells",
			Self::Inventory => "Inventory",
			Self::Description => "Description",
		}
	}

	fn page_html(&self) -> Html {
		match self {
			Self::Abilities => html!(<abilities::Page />),
			Self::Attributes => html!(<attributes::Page />),
			Self::Features => html!(),
			Self::Spells => html!(),
			Self::Inventory => html!(),
			Self::Description => html!(<crate::system::dnd5e::components::panel::Description />),
		}
	}
}
impl ToString for Page {
	fn to_string(&self) -> String {
		match self {
			Self::Abilities => "Abilities",
			Self::Attributes => "Attributes",
			Self::Features => "Features",
			Self::Spells => "Spells",
			Self::Inventory => "Inventory",
			Self::Description => "Description",
		}
		.into()
	}
}
impl FromStr for Page {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Abilities" => Ok(Self::Abilities),
			"Attributes" => Ok(Self::Attributes),
			"Features" => Ok(Self::Features),
			"Spells" => Ok(Self::Spells),
			"Inventory" => Ok(Self::Inventory),
			"Description" => Ok(Self::Description),
			v => Err(NotInList(
				v.into(),
				vec![
					"Abilities",
					"Attributes",
					"Features",
					"Spells",
					"Inventory",
					"Description",
				],
			)),
		}
	}
}

#[function_component]
fn PageSelect(props: &GeneralProp<UseStateHandle<Page>>) -> Html {
	let onchange = Callback::from({
		let handle = props.value.clone();
		move |evt: web_sys::Event| {
			let value = evt
				.select_value()
				.map(|s| Page::from_str(&s).ok())
				.flatten();
			handle.set(value.unwrap_or_default());
		}
	});
	html! {
		<select class="form-select" {onchange}>
			{EnumSet::<Page>::all().into_iter().map(|page| html! {
				<option selected={*props.value == page} value={page.to_string()}>{page.display_name()}</option>
			}).collect::<Vec<_>>()}
		</select>
	}
}

#[function_component]
pub fn Display(ViewProps { swap_view: _ }: &ViewProps) -> Html {
	let page_handle = use_state_eq(|| Page::default());

	html! {
		<div class="m-1 paged-display">
			<PageSelect value={page_handle.clone()} />
			{EnumSet::<Page>::all().into_iter().map(|page| html! {
				<div class="page" id={page.to_string()}>
					{page.page_html()}
				</div>
			}).collect::<Vec<_>>()}
		</div>
	}
}
