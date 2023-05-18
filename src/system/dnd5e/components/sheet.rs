use crate::{
	components::modal,
	system::dnd5e::{
		data::character::{ActionEffect, Character, Persistent},
		DnD5e,
	},
};
use yew::prelude::*;

mod display;
pub use display::*;
pub mod editor;

#[derive(Clone, PartialEq)]
pub struct SharedCharacter(UseReducerHandle<Character>);
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

#[derive(Clone, PartialEq, Properties)]
pub struct CharacterSheetPageProps {
	pub character: Persistent,
}

#[function_component]
pub fn CharacterSheetPage(CharacterSheetPageProps { character }: &CharacterSheetPageProps) -> Html {
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let character = SharedCharacter(use_reducer({
		let system = system.clone();
		let character = character.clone();
		let default_blocks = system.default_blocks.values().cloned().collect();
		move || Character::new(character, default_blocks)
	}));
	let modal_dispatcher = modal::Context::from(use_reducer(|| modal::State::default()));
	let show_editor = use_state_eq(|| false);

	let open_viewer = Callback::from({
		let show_editor = show_editor.clone();
		move |_| show_editor.set(false)
	});
	let open_editor = Callback::from({
		let show_editor = show_editor.clone();
		move |_| show_editor.set(true)
	});

	html! {
		<ContextProvider<SharedCharacter> context={character.clone()}>
			<ContextProvider<modal::Context> context={modal_dispatcher.clone()}>
				<div style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
					<modal::GeneralPurpose />
					{match *show_editor {
						true => html! { <editor::SheetEditor {open_viewer} /> },
						false => html! { <SheetDisplay {open_editor} /> },
					}}
				</div>
			</ContextProvider<modal::Context>>
		</ContextProvider<SharedCharacter>>
	}
}
