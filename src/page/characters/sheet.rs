use crate::{
	components::{modal, Spinner},
	database::app::{Database, FetchError},
	system::{
		self,
		core::{SourceId, System},
		dnd5e::{
			components::GeneralProp,
			data::character::{Character, DefaultsBlock, ObjectCacheProvider, Persistent},
			DnD5e,
		},
	},
	task,
};
use std::{
	rc::Rc,
	sync::{atomic::AtomicBool, Mutex},
};
use yew::prelude::*;

#[hook]
fn use_character(id: SourceId) -> CharacterHandle {
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let task_dispatch = use_context::<task::Dispatch>().unwrap();

	let state = use_state(|| CharacterState::default());
	let handle = CharacterHandle {
		database,
		system_depot,
		task_dispatch,
		state,
		is_recompiling: Rc::new(AtomicBool::new(false)),
		pending_mutations: Rc::new(Mutex::new(Vec::new())),
	};

	// Character Initialization
	if !handle.is_loaded() && !handle.is_recompiling() {
		handle.set_recompiling(true);
		wasm_bindgen_futures::spawn_local({
			let handle = handle.clone();
			let initialize_character = async move {
				let id_str = id.to_string();
				log::debug!("Initializing character from {:?}", id_str);
				let Some(persistent) = handle.database.get_typed_entry::<Persistent>(id, handle.system_depot.clone()).await? else {
					return Err(CharacterInitializationError::CharacterMissing(id_str));
				};

				let query_defaults = handle.database.clone().query_typed::<DefaultsBlock>(
					DnD5e::id(),
					handle.system_depot.clone(),
					None,
				);
				let query_result = query_defaults.await;
				let defaults_stream = query_result.map_err(|err| {
					CharacterInitializationError::DefaultsError(format!("{err:?}"))
				})?;
				let default_blocks = defaults_stream.all().await;

				let mut character = Character::new(persistent, default_blocks);
				character.recompile();
				let provider = ObjectCacheProvider {
					database: handle.database.clone(),
					system_depot: handle.system_depot.clone(),
				};
				if let Err(err) = character.update_cached_objects(provider).await {
					log::warn!("Encountered error updating cached character objects: {err:?}");
				}
				handle.state.set(CharacterState::Loaded(character));
				handle.set_recompiling(false);
				handle.process_pending_mutations();

				Ok(()) as Result<(), CharacterInitializationError>
			};
			async move {
				if let Err(err) = initialize_character.await {
					log::error!("Failed to initialize character: {err:?}");
				}
			}
		});
	}

	handle
}

#[derive(thiserror::Error, Debug)]
enum CharacterInitializationError {
	#[error("Character at key {0:?} is not in the database.")]
	CharacterMissing(String),
	#[error(transparent)]
	EntryError(#[from] FetchError),
	#[error("Defaults block query failed: {0}")]
	DefaultsError(String),
}

#[derive(Clone)]
pub struct CharacterHandle {
	database: Database,
	system_depot: system::Depot,
	task_dispatch: task::Dispatch,
	state: UseStateHandle<CharacterState>,
	is_recompiling: Rc<AtomicBool>,
	pending_mutations: Rc<Mutex<Vec<FnMutator>>>,
}
impl PartialEq for CharacterHandle {
	fn eq(&self, other: &Self) -> bool {
		self.state == other.state
	}
}
impl std::ops::Deref for CharacterHandle {
	type Target = Character;
	fn deref(&self) -> &Self::Target {
		self.state.value()
	}
}
impl AsRef<Character> for CharacterHandle {
	fn as_ref(&self) -> &Character {
		self.state.value()
	}
}
impl CharacterHandle {
	fn set_recompiling(&self, value: bool) {
		self.is_recompiling
			.store(value, std::sync::atomic::Ordering::Relaxed);
	}

	fn is_recompiling(&self) -> bool {
		self.is_recompiling
			.load(std::sync::atomic::Ordering::Relaxed)
	}

	pub fn is_loaded(&self) -> bool {
		matches!(&*self.state, CharacterState::Loaded(_))
	}
}

pub enum MutatorImpact {
	None,
	Recompile,
}

#[derive(PartialEq, Default)]
enum CharacterState {
	#[default]
	Unloaded,
	Loaded(Character),
}
impl CharacterState {
	fn value(&self) -> &Character {
		match self {
			Self::Loaded(character) => character,
			Self::Unloaded => panic!("character not loaded"),
		}
	}
}

type FnMutator = Box<dyn FnOnce(&mut Persistent) -> MutatorImpact + 'static>;
impl CharacterHandle {
	fn process_pending_mutations(&self) {
		let pending = {
			let mut pending = self.pending_mutations.lock().unwrap();
			pending.drain(..).collect::<Vec<_>>()
		};
		if pending.is_empty() {
			return;
		}

		let mut character = match &*self.state {
			CharacterState::Unloaded => {
				log::error!("Failed to apply character mutation, character is not yet initialized");
				return;
			}
			CharacterState::Loaded(character) => character.clone(),
		};

		let mut requires_recompile = false;
		for mutator in pending {
			match mutator(character.persistent_mut()) {
				MutatorImpact::None => {}
				MutatorImpact::Recompile => {
					requires_recompile = true;
				}
			}
		}
		if !requires_recompile {
			self.state.set(CharacterState::Loaded(character));
			return;
		}

		let handle = self.clone();
		self.set_recompiling(true);
		character.clear_derived();
		let signal = self
			.task_dispatch
			.spawn("Recompile Character", None, async move {
				character.recompile();
				let provider = ObjectCacheProvider {
					database: handle.database.clone(),
					system_depot: handle.system_depot.clone(),
				};
				if let Err(err) = character.update_cached_objects(provider).await {
					log::warn!("Encountered error updating cached character objects: {err:?}");
				}
				handle.state.set(CharacterState::Loaded(character));
				Ok(()) as anyhow::Result<()>
			});

		let handle = self.clone();
		wasm_bindgen_futures::spawn_local(async move {
			signal.wait_true().await;
			handle.set_recompiling(false);
			handle.process_pending_mutations();
		});
	}

	pub fn dispatch<F>(&self, mutator: F)
	where
		F: FnOnce(&mut Persistent) -> MutatorImpact + 'static,
	{
		{
			let mut pending_mutations = self.pending_mutations.lock().unwrap();
			pending_mutations.push(Box::new(mutator));
		}
		if !self.is_recompiling() {
			self.process_pending_mutations();
		}
	}

	pub fn new_dispatch<I, F>(&self, mutator: F) -> Callback<I>
	where
		I: 'static,
		F: Fn(I, &mut Persistent) -> MutatorImpact + 'static,
	{
		let handle = self.clone();
		let mutator = std::rc::Rc::new(mutator);
		Callback::from(move |input: I| {
			let mutator = mutator.clone();
			handle.dispatch(move |persistent| (*mutator)(input, persistent));
		})
	}
}

#[function_component]
pub fn Sheet(props: &GeneralProp<SourceId>) -> Html {
	let character = use_character(props.value.clone());

	let show_editor = use_state_eq(|| false);
	let open_viewer = Callback::from({
		let show_editor = show_editor.clone();
		move |_| show_editor.set(false)
	});
	let open_editor = Callback::from({
		let show_editor = show_editor.clone();
		move |_| show_editor.set(true)
	});

	if !character.is_loaded() {
		return html!(<Spinner />);
	}

	html! {<>
		<ContextProvider<CharacterHandle> context={character.clone()}>
			<div style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
				<modal::Provider>
					<modal::GeneralPurpose />
					{match *show_editor {
						true => html! { <system::dnd5e::components::editor::SheetEditor {open_viewer} /> },
						false => html! { <system::dnd5e::components::SheetDisplay {open_editor} /> },
					}}
				</modal::Provider>
			</div>
		</ContextProvider<CharacterHandle>>
	</>}
}
