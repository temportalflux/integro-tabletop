use crate::page::characters::sheet::{CharacterHandle, joined::pronouns};
use yew::prelude::*;

#[function_component]
pub fn Description() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	html! {<>
		<div>
			{"Pronouns: "}
			{pronouns(&state.persistent().description).unwrap_or_default()}
		</div>
		<div>
			{"Size: "}
			{state.persistent().description.size()}
			<div>{state.persistent().description.size().description()}</div>
		</div>
		<div>
			{"Height: "}
			{state.persistent().description.height}
			{" inches"}
		</div>
		<div>
			{"Weight: "}
			{state.persistent().description.weight}
			{" lbs"}
		</div>
		<div>
			{"Age: "}
			{state.persistent().description.age}
		</div>
		{state.persistent().description.personality.iter().map(|(kind, traits)| {
			html! {
				<div>
					<span style="font-weight: 700;">{kind}</span>
					<ul style="margin-bottom: 0;">
						{traits.iter().map(move |trait_str| {
							html!(<li>{trait_str}</li>)
						}).collect::<Vec<_>>()}
					</ul>
				</div>
			}
		}).collect::<Vec<_>>()}
		<div>
			{"Appearance: "}
			{&state.persistent().description.appearance}
		</div>
		<div>
			{"Notes: "}
			{&state.persistent().description.notes}
		</div>
	</>}
}
