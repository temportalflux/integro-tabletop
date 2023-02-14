use crate::{bootstrap::components::Tooltip, data::ContextMut, system::dnd5e::{character::State, proficiency}};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ProficiencySectionProps {
	pub title: String,
	pub kind: proficiency::Kind,
}

#[function_component]
pub fn ProficiencySection(ProficiencySectionProps { title, kind }: &ProficiencySectionProps) -> Html {
	let state = use_context::<ContextMut<State>>().unwrap();

	let items = {
		let entries = state.get_proficiencies(*kind);
		let count = entries.len();
		entries
			.iter()
			.enumerate()
			.map(|(idx, (value, srcs))| (value, srcs, idx == count - 1))
			.fold(Vec::new(), |mut html, (value, sources, is_last)| {
				let tooltip = crate::data::as_feature_paths_html(sources.iter());
				html.push(html! {
					<span>
						<Tooltip tag="span" content={tooltip} use_html={true}>
							{value.clone()}
						</Tooltip>
						{match is_last {
							false => ", ",
							true => "",
						}}
					</span>
				});
				html
			})
	};
	
	html! {
		<div style="border-style: solid; border-color: var(--bs-border-color); border-width: 0; border-bottom-width: var(--bs-border-width);">
			<h6 style="font-size: 0.8rem; color: var(--bs-card-title-color);">{title.clone()}</h6>
			<span>{match items.is_empty() {
				true => html! { {"None"} },
				false => html! {<> {items} </>},
			}}</span>
		</div>
	}
}

#[function_component]
pub fn Proficiencies() -> Html {
	html! {
		<div id="proficiencies-container" class="card" style="max-width: 200px; margin: 0 auto; border-color: var(--theme-frame-color);">
			<div class="card-body" style="padding: 5px;">
				<h5 class="card-title text-center" style="font-size: 0.8rem;">{"Proficiencies"}</h5>
				<ProficiencySection title={"Languages"} kind={proficiency::Kind::Language} />
				<ProficiencySection title={"Armor"} kind={proficiency::Kind::Armor} />
				<ProficiencySection title={"Weapons"} kind={proficiency::Kind::Weapon} />
				<ProficiencySection title={"Tools"} kind={proficiency::Kind::Tool} />
			</div>
		</div>
	}
}
