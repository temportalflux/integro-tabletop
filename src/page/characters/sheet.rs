use crate::{
	components::{database, mobile, Spinner, ViewScaler},
	storage::autosync,
	system::{core::SourceId, dnd5e::components::GeneralProp},
};
use yew::prelude::*;

mod handle;
pub use handle::*;
pub mod joined;
pub mod paged;

#[derive(Clone, Copy, PartialEq, Debug)]
enum View {
	Display,
	Editor,
}

#[derive(Clone, PartialEq, Properties)]
pub struct ViewProps {
	pub swap_view: Callback<()>,
}

#[hook]
fn use_update_character_modules(character: &CharacterHandle) {
	let is_first_mount_after_load = use_mut_ref(|| true);
	let autosync_channel = use_context::<autosync::Channel>().unwrap();
	let modules_query = database::use_query_modules(None);
	let modules_query_complete = matches!(modules_query.status(), database::QueryStatus::Success(_));
	if character.is_loaded() && modules_query_complete && *is_first_mount_after_load.borrow_mut() {
		*is_first_mount_after_load.borrow_mut() = false;

		let module_ids = match modules_query.status() {
			database::QueryStatus::Success(data) => data.iter().map(|module| module.id.clone()).collect::<Vec<_>>(),
			_ => Vec::new(),
		};

		autosync_channel.try_send_req(autosync::Request::UpdateModules(module_ids));
	}
}

#[function_component]
pub fn Sheet(props: &GeneralProp<SourceId>) -> Html {
	let character = use_character(props.value.clone());
	use_update_character_modules(&character);

	let autosync_channel = use_context::<autosync::Channel>().unwrap();
	crate::components::hook::use_document_visibility({
		let character = character.clone();
		let autosync_channel = autosync_channel.clone();
		move |vis| {
			if vis == web_sys::VisibilityState::Visible && character.is_loaded() {
				autosync_channel.try_send_req(autosync::Request::UpdateFile(character.id().clone()));
			}
		}
	});

	let screen_size = mobile::use_mobile_kind();
	let view_handle = use_state_eq({
		let is_new = !props.value.has_path();
		move || match is_new {
			true => View::Editor,
			false => View::Display,
		}
	});
	let swap_view = Callback::from({
		let view_handle = view_handle.clone();
		move |_| {
			view_handle.set(match *view_handle {
				View::Display => View::Editor,
				View::Editor => View::Display,
			});
		}
	});

	use_effect_with(props.value.clone(), {
		let character = character.clone();
		move |id: &SourceId| {
			if character.is_loaded() {
				log::info!("Reloading character with updated id {id:?}");
				character.unload();
			}
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
	html! {
		<ContextProvider<CharacterHandle> context={character.clone()}>
			<div class="w-100 h-100" style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
				<div class="page-root">
					<ViewScaler ranges={vec![(1200.0..1400.0).into(), (1400.0..).into()]}>
						{content}
					</ViewScaler>
				</div>
				<crate::components::context_menu::ContextMenu />
			</div>
		</ContextProvider<CharacterHandle>>
	}
}
