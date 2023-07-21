use crate::{
	components::{mobile, modal, Spinner, ViewScaler},
	system::{core::SourceId, dnd5e::components::GeneralProp},
};
use yew::prelude::*;

mod handle;
pub use handle::*;
pub mod joined;
pub mod paged;

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

	let screen_size = mobile::use_mobile_kind();
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

	let content = match (screen_size, *view_handle) {
		(mobile::Kind::Desktop, View::Display) => {
			html!(<joined::Display {swap_view} />)
		}
		(mobile::Kind::Desktop, View::Editor) => {
			html!(<joined::editor::Editor {swap_view} />)
		}
		(mobile::Kind::Mobile, View::Display) => {
			html!(<paged::Display {swap_view} />)
		}
		(mobile::Kind::Mobile, View::Editor) => {
			html!("Paged Editor TODO")
		}
	};
	html! {<>
		<ContextProvider<CharacterHandle> context={character.clone()}>
			<div class="page-root" style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
				<ViewScaler ranges={vec![(1200.0..1400.0).into(), (1400.0..).into()]}>
					{content}
				</ViewScaler>
			</div>
		</ContextProvider<CharacterHandle>>
	</>}
}
