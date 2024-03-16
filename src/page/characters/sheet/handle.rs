use crate::{
	database::{Database, FetchError},
	system::{
		self,
		dnd5e::data::character::{Character, DefaultsBlock, ObjectCacheProvider, Persistent},
		SourceId,
	},
	task,
};
use std::{
	rc::Rc,
	sync::{atomic::AtomicBool, Mutex},
};
use yew::prelude::*;

#[hook]
pub fn use_character(id: SourceId) -> CharacterHandle {
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Registry>().unwrap();
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
	use_effect_with(
		(handle.clone(), handle.is_loaded(), handle.is_recompiling()),
		|(handle, is_loaded, is_recompiling)| {
			if !is_recompiling && !is_loaded {
				handle.set_recompiling(true);
				handle.load_with(id);
			}
		},
	);

	handle
}

#[derive(thiserror::Error, Debug)]
enum CharacterInitializationError {
	#[error("Character has no game system associated with it.")]
	NoSystem,
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
	system_depot: system::Registry,
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
		self.is_recompiling.store(value, std::sync::atomic::Ordering::Relaxed);
	}

	fn is_recompiling(&self) -> bool {
		self.is_recompiling.load(std::sync::atomic::Ordering::Relaxed)
	}

	pub fn is_loaded(&self) -> bool {
		matches!(&*self.state, CharacterState::Loaded(_))
	}

	pub fn unload(&self) {
		self.state.set(CharacterState::Unloaded);
	}

	fn load_with(&self, id: SourceId) {
		wasm_bindgen_futures::spawn_local({
			let handle = self.clone();
			let initialize_character = async move {
				let Some(system) = &id.system else {
					return Err(CharacterInitializationError::NoSystem);
				};
				let id_str = id.to_string();
				log::info!(target: "character", "Initializing from {:?}", id_str);

				let entry = handle
					.database
					.get_typed_entry::<Persistent>(id.clone(), handle.system_depot.clone(), None)
					.await?;
				let persistent = match entry {
					Some(known) => known,
					None if !id.has_path() => Persistent {
						id: id.clone(),
						..Default::default()
					},
					None => {
						return Err(CharacterInitializationError::CharacterMissing(id_str));
					}
				};

				let query_defaults = handle.database.clone().query_typed::<DefaultsBlock>(
					system.as_str(),
					handle.system_depot.clone(),
					None,
				);
				let query_result = query_defaults.await;
				let defaults_stream =
					query_result.map_err(|err| CharacterInitializationError::DefaultsError(format!("{err:?}")))?;
				let default_blocks = defaults_stream.all().await;
				let default_blocks = default_blocks.into_iter().map(|(_, block)| block).collect();

				let mut character = Character::new(persistent, default_blocks);
				let provider = ObjectCacheProvider {
					database: handle.database.clone(),
					system_depot: handle.system_depot.clone(),
				};
				if let Err(err) = character.recompile(provider).await {
					log::warn!(target: "character", "Encountered error updating cached character objects: {err:?}");
				}
				log::info!(target: "character", "Finished loading {:?}", id_str);
				handle.state.set(CharacterState::Loaded(character));
				handle.set_recompiling(false);
				handle.process_pending_mutations();

				Ok(()) as Result<(), CharacterInitializationError>
			};
			async move {
				if let Err(err) = initialize_character.await {
					log::error!(target: "character", "Failed to initialize character: {err:?}");
				}
			}
		});
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
		let signal = self.task_dispatch.spawn("Recompile Character", None, async move {
			let provider = ObjectCacheProvider {
				database: handle.database.clone(),
				system_depot: handle.system_depot.clone(),
			};
			if let Err(err) = character.recompile(provider).await {
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
