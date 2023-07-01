use crate::{
	components::{modal, use_media_query, Spinner},
	system::{core::SourceId, dnd5e::components::GeneralProp},
};
use yew::prelude::*;

mod handle;
pub use handle::*;
pub mod joined;
pub mod paged;

#[derive(Clone, Copy, PartialEq)]
enum Presentation {
	Joined,
	Paged,
}
#[derive(Clone, Copy, PartialEq)]
enum View {
	Display,
	Editor,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ViewProps {
	pub swap_view: Callback<()>,
}

#[function_component]
pub fn Sheet(props: &GeneralProp<SourceId>) -> Html {
	let character = use_character(props.value.clone());

	let is_full_page = use_media_query("(min-width: 1200px)");
	let presentation = match *is_full_page {
		true => Presentation::Joined,
		false => Presentation::Paged,
	};
	let view_handle = use_state_eq(|| View::Display);
	let swap_view = Callback::from({
		let view_handle = view_handle.clone();
		move |_| {
			view_handle.set(match *view_handle {
				View::Display => View::Editor,
				View::Editor => View::Display,
			});
		}
	});

	if !character.is_loaded() {
		return html!(<Spinner />);
	}

	let content = match (presentation, *view_handle) {
		(Presentation::Joined, View::Display) => {
			html!(<joined::Display {swap_view} />)
		}
		(Presentation::Joined, View::Editor) => {
			html!(<joined::editor::Editor {swap_view} />)
		}
		(Presentation::Paged, View::Display) => {
			html!(<paged::Display {swap_view} />)
		}
		(Presentation::Paged, View::Editor) => {
			html!("Paged Editor TODO")
		}
	};
	html! {<>
		<ContextProvider<CharacterHandle> context={character.clone()}>
			<div style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
				<modal::Provider>
					<modal::GeneralPurpose />
					{content}
				</modal::Provider>
			</div>
		</ContextProvider<CharacterHandle>>
	</>}
}
