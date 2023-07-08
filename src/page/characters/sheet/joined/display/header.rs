use crate::page::characters::sheet::CharacterHandle;
use itertools::Itertools;
use yew::prelude::*;

#[function_component]
pub fn Header() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();

	let description = &state.persistent().description;
	let pronouns = pronouns(description).map(|items| {
		html! {
			<span class="pronouns ms-1">{"("}{items}{")"}</span>
		}
	});
	let name = html! {
		<span class="identity">
			<span class="name">{&description.name}</span>
			{pronouns.unwrap_or_default()}
		</span>
	};

	let mut races = Vec::new();
	let mut race_variants = Vec::new();
	let mut lineages = Vec::new();
	let mut upbringings = Vec::new();
	let mut backgrounds = Vec::new();
	for bundle in &state.persistent().bundles {
		match bundle.category.as_str() {
			"Race" => races.push(bundle.name.as_str()),
			"RaceVariant" => race_variants.push(bundle.name.as_str()),
			"Lineage" => lineages.push(bundle.name.as_str()),
			"Upbringing" => upbringings.push(bundle.name.as_str()),
			"Background" => backgrounds.push(bundle.name.as_str()),
			_ => {}
		}
	}

	let race = group_names(html!(", "), races).map(|items| {
		html! {
			<>{items}</>
		}
	});
	let race_variant = group_names(html!(", "), race_variants).map(|items| {
		html! {
			<span class="ms-1">{"("}{items}{")"}</span>
		}
	});
	let race = race.map(|race| {
		html! {
			<div class="group race">
				{"Race: "}
				{race}
				{race_variant.unwrap_or_default()}
			</div>
		}
	});

	let lineage =
		group_names(html!(" / "), lineages).map(|items| html!(<span class="mx-1">{items}</span>));
	let upbringing = group_names(html!(" / "), upbringings)
		.map(|items| html!(<span class="mx-1">{items}</span>));
	let joiner = (lineage.is_some() && upbringing.is_some()).then_some(html!("&"));
	let lineage_upbringing = (lineage.is_some() || upbringing.is_some()).then(|| {
		html! {
			<div class="group lineage">
				{"Lineage & Upbringing:"}
				{lineage.unwrap_or_default()}
				{joiner.unwrap_or_default()}
				{upbringing.unwrap_or_default()}
			</div>
		}
	});

	let background = group_names(html!(", "), backgrounds).map(|items| {
		html! {
			<div class="group background">{"Background: "}{items}</div>
		}
	});

	let total_level = state.level(None);
	let classes = state
		.persistent()
		.classes
		.iter()
		.map(|class| html!(format!("{} {}", class.name, class.current_level)));
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

pub fn pronouns(
	description: &crate::system::dnd5e::data::character::Description,
) -> Option<Vec<Html>> {
	let iter_pronouns = description.pronouns.iter().sorted();
	let iter_pronouns = iter_pronouns.chain(match description.custom_pronouns.is_empty() {
		true => vec![],
		false => vec![&description.custom_pronouns],
	});
	group_names(html!(", "), iter_pronouns)
}

fn group_names<T: AsRef<str>>(
	separator: Html,
	iter: impl IntoIterator<Item = T>,
) -> Option<Vec<Html>> {
	let iter = iter.into_iter().map(|name| html!(name.as_ref()));
	let items = Itertools::intersperse(iter, separator).collect::<Vec<_>>();
	(!items.is_empty()).then(|| items)
}
