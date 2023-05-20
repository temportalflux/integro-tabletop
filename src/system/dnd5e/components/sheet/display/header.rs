use itertools::Itertools;
use yew::prelude::*;
use crate::system::dnd5e::components::SharedCharacter;

#[function_component]
pub fn Header() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	let description = &state.persistent().description;
	let iter_pronouns = description.pronouns.iter().sorted();
	let iter_pronouns = iter_pronouns.chain(match description.custom_pronouns.is_empty() {
		true => vec![],
		false => vec![&description.custom_pronouns],
	});
	let pronouns = group_names(html!(", "), iter_pronouns).map(|items| html! {
		<span class="pronouns ms-1">{"("}{items}{")"}</span>
	});
	let name = html! {
		<span class="identity">
			<span class="name">{&description.name}</span>
			{pronouns.unwrap_or_default()}
		</span>
	};

	let named_groups = &state.persistent().named_groups;

	let race = named_groups.race.iter().map(|var| &var.name);
	let race = group_names(html!(", "), race).map(|items| html! {
		<>{items}</>
	});
	let race_variant = named_groups.race_variant.iter().map(|var| &var.name);
	let race_variant = group_names(html!(", "), race_variant).map(|items| html! {
		<span class="ms-1">{"("}{items}{")"}</span>
	});
	let race = race.map(|race| html! {
		<div class="group race">
			{"Race: "}
			{race}
			{race_variant.unwrap_or_default()}
		</div>
	});

	let upbringing = named_groups.upbringing.iter().map(|gp| &gp.name);
	let upbringing = group_names(html!(" / "), upbringing).map(|items| html! {
		<>
			<span class="ms-1 me-1">{items}</span>
			{"&"}
		</>
	});
	let lineage = named_groups.lineage.iter().map(|gp| &gp.name);
	let lineage = group_names(html!(" / "), lineage).map(|items| html! {
		<span class="ms-1">{items}</span>
	});
	let lineage_upbringing = (lineage.is_some() || upbringing.is_some()).then(|| html! {
		<div class="group lineage">
			{"Lineage & Upbringing: "}
			{upbringing.unwrap_or_default()}
			{lineage.unwrap_or_default()}
		</div>
	});

	let background = named_groups.background.iter().map(|bg| &bg.name);
	let background = group_names(html!(", "), background).map(|items| html! {
		<div class="group background">{"Background: "}{items}</div>
	});
	
	let total_level = state.level(None);
	let classes = state.persistent().classes.iter().map(|class| {
		html!(format!("{} {}", class.name, class.levels.len()))
	});
	let classes = Itertools::intersperse(classes, html!(" / ")).collect::<Vec<_>>();
	
	html! {
		<div class="sheet-header">
			{name}
			{race}
			{lineage_upbringing}
			{background.unwrap_or_default()}
			<div class="level">{format!("Character Level ({total_level}): ")}{classes}</div>
		</div>
	}
}

fn group_names<'a>(separator: Html, iter: impl Iterator<Item=&'a String>) -> Option<Vec<Html>> {
	let iter = iter.map(|name| html!(name));
	let items = Itertools::intersperse(iter, separator).collect::<Vec<_>>();
	(!items.is_empty()).then(|| items)
}
