use std::rc::Rc;

use crate::{
	database::app::Database,
	system::{
		self,
		core::{SourceId, System},
		dnd5e::{
			components::{GeneralProp},
			data::character::{DefaultsBlock, Persistent, Character, ActionEffect},
			DnD5e,
		},
	}, components::modal,
};
use futures_util::StreamExt;
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncOptions};

#[derive(Clone, PartialEq)]
pub struct SharedCharacter(pub(crate) UseReducerHandle<Character>); // TODO: doesnt need to be pub
impl std::ops::Deref for SharedCharacter {
	type Target = UseReducerHandle<Character>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl AsRef<Character> for SharedCharacter {
	fn as_ref(&self) -> &Character {
		&*self.0
	}
}
impl SharedCharacter {
	pub fn new_dispatch<I, F>(&self, mutator: F) -> Callback<I>
	where
		I: 'static,
		F: Fn(I, &mut Persistent, &std::rc::Rc<Character>) -> Option<ActionEffect> + 'static,
	{
		let handle = self.0.clone();
		let mutator = std::rc::Rc::new(mutator);
		Callback::from(move |input: I| {
			let mutator = mutator.clone();
			handle.dispatch(Box::new(move |a, b| (*mutator)(input, a, b)));
		})
	}
}

#[function_component]
pub fn Sheet(props: &GeneralProp<SourceId>) -> Html {
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let character = SharedCharacter(use_reducer(|| Character::new(Persistent::default(), Vec::new())));
	let initiaize_character = use_async_with_options(
		{
			let character = character.clone();
			let id = props.value.clone();
			async move {
				let Some(persistent) = database.get_typed_entry::<Persistent>(id.clone(), system_depot.clone()).await? else {
					return Err(QueryCharacterError::EntryMissing(id.to_string()));
				};

				let default_blocks = database
					.query_typed::<DefaultsBlock>(DnD5e::id(), system_depot.clone(), None)
					.await?
					.all()
					.await;

				character.dispatch(Box::new(move |_: &mut Persistent, _: &Rc<Character>| {
					Some(ActionEffect::Reset(persistent, default_blocks))
				}));

				Ok(()) as Result<(), QueryCharacterError>
			}
		},
		UseAsyncOptions::enable_auto(),
	);
	// TODO: Remove dependence on system struct
	let system = use_state(|| DnD5e::default());
	
	let show_editor = use_state_eq(|| false);
	let open_viewer = Callback::from({
		let show_editor = show_editor.clone();
		move |_| show_editor.set(false)
	});
	let open_editor = Callback::from({
		let show_editor = show_editor.clone();
		move |_| show_editor.set(true)
	});

	if initiaize_character.loading || initiaize_character.data.is_none() {
		return html! {
			<div class="spinner-border" role="status">
				<span class="visually-hidden">{"Loading..."}</span>
			</div>
		};
	}

	log::debug!("{:?}", character.persistent().feats);

	html! {<>
		<ContextProvider<UseStateHandle<DnD5e>> context={system}>
			<ContextProvider<SharedCharacter> context={character.clone()}>
				<div style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
					<modal::Provider>
						<modal::GeneralPurpose />
						{match *show_editor {
							true => html! { <system::dnd5e::components::editor::SheetEditor {open_viewer} /> },
							false => html! { <system::dnd5e::components::SheetDisplay {open_editor} /> },
						}}
					</modal::Provider>
				</div>
			</ContextProvider<SharedCharacter>>
		</ContextProvider<UseStateHandle<DnD5e>>>
	</>}
}

#[derive(thiserror::Error, Debug, Clone)]
enum QueryCharacterError {
	#[error("Entry at key {0:?} is not in the database.")]
	EntryMissing(String),
	#[error(transparent)]
	DatabaseError(#[from] crate::database::Error),
}
