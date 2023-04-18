use crate::kdl_ext::NodeContext;
use anyhow::Context;
use futures_util::StreamExt;
use multimap::MultiMap;
use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
};
use yew::prelude::*;

pub mod bootstrap;
pub mod components;
pub mod data;
pub mod database;
pub mod kdl_ext;
pub mod logging;
pub mod path_map;
pub mod system;
pub mod theme;
pub mod utility;

#[derive(Clone, PartialEq)]
pub struct Compiled<T> {
	wrapped: T,
	update_root_channel: UseStateHandle<()>,
}
impl<T> std::ops::Deref for Compiled<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.wrapped
	}
}
impl<T> Compiled<T> {
	pub fn update_root(&self) {
		self.update_root_channel.set(());
	}
}

fn create_character(system: &system::dnd5e::DnD5e) -> system::dnd5e::data::character::Persistent {
	use system::{
		core::SourceId,
		dnd5e::data::{
			character::{Description, Persistent},
			currency::{self, Wallet},
			Ability,
		},
	};
	let mut persistent = Persistent {
		description: Description {
			name: "Sid the Squid".into(),
			..Default::default()
		},
		..Default::default()
	};
	persistent.classes.push({
		let mut class = system
			.classes
			.get(&SourceId::from_str("local://basic-rules@dnd5e/class/barbarian.kdl").unwrap())
			.unwrap()
			.clone();
		class.levels.truncate(3); // level 3
		class
	});
	persistent.named_groups.background.push(
		system
			.backgrounds
			.get(&SourceId::from_str("local://basic-rules@dnd5e/background/folkHero.kdl").unwrap())
			.unwrap()
			.clone(),
	);
	persistent.named_groups.lineage.push(
		system
			.lineages
			.get(
				&SourceId::from_str("local://elf-and-orc@dnd5e/lineage/dwarven/dwarven.kdl")
					.unwrap(),
			)
			.unwrap()
			.clone(),
	);
	persistent.named_groups.lineage.push(
		system
			.lineages
			.get(&SourceId::from_str("local://elf-and-orc@dnd5e/lineage/dwarven/gray.kdl").unwrap())
			.unwrap()
			.clone(),
	);
	persistent.named_groups.upbringing.push(
		system
			.upbringings
			.get(&SourceId::from_str("local://elf-and-orc@dnd5e/upbringing/abjurer.kdl").unwrap())
			.unwrap()
			.clone(),
	);
	persistent.ability_scores[Ability::Strength] = 15;
	persistent.ability_scores[Ability::Dexterity] = 10;
	persistent.ability_scores[Ability::Constitution] = 14;
	persistent.ability_scores[Ability::Intelligence] = 12;
	persistent.ability_scores[Ability::Wisdom] = 8;
	persistent.ability_scores[Ability::Charisma] = 13;
	persistent.hit_points.current = 6;
	*persistent.inventory.wallet_mut() = Wallet::from([
		(3, currency::Kind::Platinum),
		(16, currency::Kind::Gold),
		(4, currency::Kind::Electrum),
		(30, currency::Kind::Silver),
		(152, currency::Kind::Copper),
	]);
	persistent
}

#[function_component]
fn App() -> Html {
	use database::app::Database;
	use system::dnd5e::{self, DnD5e};
	let show_browser = use_state_eq(|| false);
	let comp_reg = use_memo(|_| dnd5e::component_registry(), ());
	let node_reg = Arc::new(dnd5e::node_registry());
	let system = use_state(|| DnD5e::default());
	let database = yew_hooks::use_async(async move {
		match Database::open().await {
			Ok(db) => Ok(db),
			Err(err) => {
				log::error!(target: "tabletop-tools", "Failed to connect to database: {err:?}");
				Err(Arc::new(err))
			}
		}
	});
	let load_modules = yew_hooks::use_async({
		let database = database.clone();
		async move {
			let Some(database) = &database.data else { return Err(()); };
			log::debug!(target: "tabletop-tools", "Loading modules into database");
			if let Err(err) = load_modules(database).await {
				log::error!(target: "tabletop-tools", "Failed to load modules into database: {err:?}");
				return Err(());
			}
			Ok(())
		}
	});
	let load_content = yew_hooks::use_async({
		let database = database.clone();
		let comp_reg = comp_reg.clone();
		let node_reg = node_reg.clone();
		let system_state = system.clone();
		async move {
			use database::{
				app::{entry, Entry},
				ObjectStoreExt, TransactionExt,
			};
			let Some(database) = &database.data else { return Err(LoadContentError::NoDatabase); };
			log::debug!(target: "tabletop-tools", "Loading content from database");
			let transaction = database.read_entries()?;
			let entries_store = transaction.object_store_of::<Entry>()?;
			let idx_by_system = entries_store.index_of::<entry::System>()?;
			let query = entry::System {
				system: "dnd5e".into(),
			};

			let mut system = DnD5e::default();
			let mut cursor = idx_by_system.open_cursor(Some(&query)).await?;
			while let Some(entry) = cursor.next().await {
				let source_id = entry.source_id();
				let document = match entry.kdl.parse::<kdl::KdlDocument>() {
					Ok(doc) => doc,
					Err(err) => {
						log::error!(target: "tabletop-tools", "Failed to parse the contents of {:?}: {err:?}", source_id.to_string());
						continue;
					}
				};
				for node in document.nodes() {
					let Some(comp_factory) = comp_reg.get_factory(node.name().value()).cloned() else { continue; };
					let ctx = NodeContext::new(Arc::new(source_id.clone()), node_reg.clone());
					match comp_factory
						.add_from_kdl(node, &ctx)
						.with_context(|| format!("Failed to parse {:?}", source_id.to_string()))
					{
						Ok(insert_callback) => {
							(insert_callback)(&mut system);
						}
						Err(err) => {
							log::error!(target: &entry.module, "{err:?}");
						}
					}
				}
			}
			system_state.set(system);
			Ok(()) as Result<(), LoadContentError>
		}
	});

	// When the app first opens, load the database.
	// Could probably check `use_is_first_mount()`, but checking if there database
	// doesn't exist yet and isn't loading is more clear.
	if database.data.is_none() && !database.loading {
		database.run();
	}
	// Once the database is connected, load the modules into it.
	use_effect_with_deps(
		{
			let load_modules = load_modules.clone();
			move |_db| {
				load_modules.run();
			}
		},
		database.data.clone(),
	);
	use_effect_with_deps(
		{
			let load_content = load_content.clone();
			move |_db| {
				load_content.run();
			}
		},
		load_modules.data.clone(),
	);

	let initial_character = use_state(|| None);
	use_effect_with_deps(
		{
			let initial_character = initial_character.clone();
			move |(system, has_loaded): &(UseStateHandle<DnD5e>, bool)| {
				if *has_loaded {
					initial_character.set(Some(create_character(&**system)));
				}
			}
		},
		(system.clone(), load_content.data.is_some()),
	);

	let open_character = Callback::from({
		let show_browser = show_browser.clone();
		move |_| {
			show_browser.set(false);
		}
	});
	let open_content = Callback::from({
		let show_browser = show_browser.clone();
		move |_| {
			show_browser.set(true);
		}
	});

	let content = match *show_browser {
		false => match &*initial_character {
			None => html! {},
			Some(character) => {
				let character = character.clone();
				html! {<system::dnd5e::components::CharacterSheetPage {character} />}
			}
		},
		true => html! {},
	};

	return html! {<>
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<a class="navbar-brand" href="/">{"Tabletop Tools"}</a>
					<button
						class="navbar-toggler" type="button"
						data-bs-toggle="collapse" data-bs-target="#navContent"
						aria-controls="navContent" aria-expanded="false" aria-label="Toggle navigation"
					>
						<span class="navbar-toggler-icon"></span>
					</button>
					<div class="collapse navbar-collapse" id="navContent">
						<ul class="navbar-nav">
							<li class="nav-item">
								<a class="nav-link" onclick={open_character}>{"My Characters"}</a>
							</li>
							<li class="nav-item">
								<a class="nav-link" onclick={open_content}>{"Content Browser"}</a>
							</li>
						</ul>
						<ul class="navbar-nav flex-row flex-wrap ms-md-auto">
							<theme::Dropdown />
						</ul>
					</div>
				</div>
			</nav>
		</header>
		<ContextProvider<Option<Database>> context={database.data.clone()}>
			<ContextProvider<UseStateHandle<DnD5e>> context={system.clone()}>
				{content}
			</ContextProvider<UseStateHandle<DnD5e>>>
		</ContextProvider<Option<Database>>>
	</>};
}

#[cfg(target_family = "wasm")]
fn main() {
	logging::wasm::init(logging::wasm::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

async fn load_modules(database: &database::app::Database) -> Result<(), database::Error> {
	use database::{
		app::{module::NameSystem, Entry, Module},
		ObjectStoreExt, TransactionExt,
	};
	use system::core::ModuleId;
	let known_modules = vec![
		("basic-rules", vec!["dnd5e"]),
		("elf-and-orc", vec!["dnd5e"]),
		("wotc-oota", vec!["dnd5e"]),
		//("wotc-phb", vec!["dnd5e"]),
		("wotc-toa", vec!["dnd5e"]),
	];
	let mut systems_to_load = MultiMap::<&'static str, &'static str>::new();
	// Find all the missing modules (if any)
	{
		let read_only = database.read_modules()?;
		let store = read_only.object_store_of::<Module>()?;
		let index = store.index_of::<NameSystem>()?;
		let mut params = NameSystem::default();
		for (module_name, systems) in known_modules {
			params.name = module_name.to_owned();
			for system in systems {
				params.system = system.to_owned();
				if index.get(&params).await?.is_none() {
					systems_to_load.insert(module_name, system);
				}
			}
		}
	}
	// Fetch the document data for all module-systems not yet in the database
	for (module_name, systems) in systems_to_load.into_iter() {
		log::info!("Loading content module {module_name:?}");

		let sources = match fetch_local_module(module_name, &systems[..]).await {
			Ok(sources) => sources,
			Err(err) => {
				log::error!("Failed to fetch module {module_name:?}: {err:?}");
				continue;
			}
		};

		let transaction = database.write()?;
		let module_store = transaction.object_store_of::<Module>()?;
		let entry_store = transaction.object_store_of::<Entry>()?;
		for system in &systems {
			let record = Module {
				module_id: ModuleId::Local {
					name: module_name.into(),
				},
				name: module_name.into(),
				system: (*system).into(),
			};
			module_store.add_record(&record).await?;
		}

		for (mut source_id, content) in sources {
			let Ok(document) = content.parse::<kdl::KdlDocument>() else { continue; };
			for (idx, node) in document.nodes().iter().enumerate() {
				source_id.node_idx = idx;
				let category = match node.name().value() {
					"condition" => "condition".into(),
					"item" => "item".into(),
					"spell" => "spell".into(),
					"feat" => "feat".into(),
					"class" => "class".into(),
					"subclass" => "subclass".into(),
					"background" => "background".into(),
					"race" => "race".into(),
					"subrace" => "race-variant".into(),
					"lineage" => "lineage".into(),
					"upbringing" => "upbringing".into(),
					"defaults" => "defaults".into(),
					name => {
						log::warn!("Unsupported category name {name:?}, cannot load into database.");
						continue;
					}
				};
				let system = source_id.system.clone().unwrap();
				let version = source_id.version.take();
				let record = Entry {
					id: source_id.to_string(),
					module: module_name.into(),
					system: system,
					category: category,
					version: version.clone(),
					kdl: node.to_string(),
				};
				source_id.version = version;
				entry_store.put_record(&record).await?;
			}
		}

		transaction.commit().await?;
	}
	Ok(())
}

#[derive(thiserror::Error, Debug, Clone)]
enum LoadContentError {
	#[error("No database connection")]
	NoDatabase,
	#[error(transparent)]
	DatabaseError(#[from] database::Error),
}

#[cfg(target_family = "windows")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	use crate::system::{
		core::{ModuleId, SourceId},
		dnd5e,
	};
	use anyhow::Context;
	use std::{collections::BTreeMap, sync::Arc};

	let _ = logging::console::init("tabletop-tools", &[]);

	let comp_reg = dnd5e::component_registry();
	let node_reg = Arc::new(dnd5e::node_registry());
	let mut system = dnd5e::DnD5e::default();

	let mut modules = Vec::new();
	for entry in std::fs::read_dir("./modules")? {
		let entry = entry?;
		if !entry.metadata()?.is_dir() {
			continue;
		}
		let module_id = entry.file_name().to_str().unwrap().to_owned();
		let mut system_ids = Vec::new();
		for entry in std::fs::read_dir(entry.path())? {
			let entry = entry?;
			if !entry.metadata()?.is_dir() {
				continue;
			}
			let system_id = entry.file_name().to_str().unwrap().to_owned();
			system_ids.push(system_id);
		}
		log::debug!("Found module {module_id:?} with systems {system_ids:?}.");
		modules.push((entry.path(), module_id, system_ids));
	}

	let mut sources = BTreeMap::new();
	for (module_path, module_id, system_ids) in modules {
		for system_id in system_ids {
			log::info!("Loading module \"{module_id}/{system_id}\"");
			let system_path = module_path.join(&system_id);
			let mut item_paths = Vec::new();
			for item in WalkDir::new(&system_path) {
				let Some(ext) = item.extension() else { continue; };
				if ext.to_str() != Some("kdl") {
					continue;
				}
				let Ok(content) = std::fs::read_to_string(&item) else { continue; };
				let item_relative_path = item.strip_prefix(&system_path)?;
				item_paths.push(item_relative_path.to_owned());
				let source_id = SourceId {
					module: Some(ModuleId::Local {
						name: module_id.clone(),
					}),
					system: Some(system_id.clone()),
					path: item_relative_path.to_owned(),
					version: None,
					node_idx: 0,
				};
				sources.insert(source_id, content);
			}
			// Update the index file
			tokio::fs::write(system_path.join("index"), {
				item_paths
					.into_iter()
					.map(|path| path.display().to_string().replace("\\", "/"))
					.collect::<Vec<_>>()
					.join("\n")
			})
			.await?;
		}
	}

	for (mut source_id, content) in sources {
		let document = content
			.parse::<kdl::KdlDocument>()
			.with_context(|| format!("Invalid KDL format in {:?}", source_id.to_string()))?;
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let node_name = node.name().value();
			let Some(comp_factory) = comp_reg.get_factory(node_name).cloned() else {
				log::error!("Failed to find factory to deserialize node \"{node_name}\".");
				continue;
			};
			let ctx = kdl_ext::NodeContext::new(Arc::new(source_id.clone()), node_reg.clone());
			let insert_parsed = comp_factory.add_from_kdl(node, &ctx);
			let insert_parsed =
				insert_parsed.with_context(|| format!("while parsing {}", source_id.to_string()));
			match insert_parsed {
				Ok(insert_callback) => (insert_callback)(&mut system),
				Err(err) => {
					log::error!("Failed to deserialize entry: {err:?}");
				}
			}
		}
	}

	Ok(())
}

async fn fetch_local_module(
	name: &'static str,
	systems: &[&'static str],
) -> anyhow::Result<BTreeMap<system::core::SourceId, String>> {
	use crate::system::core::{ModuleId, SourceId};
	// This is a temporary flow until github oauth is up and running.
	// Expects:
	// - a provided module with the folder-name `name` is at `./modules/{name}`.
	// - a module has system directories as root folders.
	//   e.g. all files for `dnd5e` live at `./modules/{name}/dnd5e/..`
	// - each system has an index file at `./modules/{name}/{system}/index`,
	//   where each line in the file is a relative path from the system folder to the resource.
	//   (This will not be needed when github fetching is up and running.)
	static ROOT_URL: &'static str = "http://localhost:8080/modules";
	let mut content = BTreeMap::new();
	for system in systems {
		let index_uri = format!("{ROOT_URL}/{name}/{system}/index");
		let Ok(resp) = reqwest::get(index_uri).await
		else {
			return Err(MissingResource(name, system, "index".into()).into());
		};
		let index_content = resp.text().await?;
		for line in index_content.lines() {
			let uri = format!("{ROOT_URL}/{name}/{system}/{line}");
			let Ok(resp) = reqwest::get(uri.clone()).await else {
				return Err(MissingResource(name, system, line.into()).into());
			};
			let source_id = SourceId {
				module: Some(ModuleId::Local {
					name: name.to_owned(),
				}),
				system: Some((*system).to_owned()),
				path: PathBuf::from(line),
				version: None,
				node_idx: 0,
			};
			content.insert(source_id, resp.text().await?);
		}
	}
	Ok(content)
}

#[derive(thiserror::Error, Debug)]
#[error("Missing resource in module {0} for system {1} at path {2}.")]
struct MissingResource(&'static str, &'static str, String);

#[derive(thiserror::Error, Debug)]
pub struct GeneralError(pub String);
impl std::fmt::Display for GeneralError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

struct WalkDir {
	iter: Option<std::fs::ReadDir>,
	stack: Vec<std::fs::ReadDir>,
}
impl WalkDir {
	fn new(path: impl AsRef<Path>) -> Self {
		Self {
			iter: std::fs::read_dir(path).ok(),
			stack: Vec::new(),
		}
	}
}
impl Iterator for WalkDir {
	type Item = PathBuf;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(mut iter) = self.iter.take() else { return None; };
			let Some(item) = iter.next() else {
				// current entry has finished
				self.iter = self.stack.pop();
				continue;
			};
			let Ok(entry) = item else {
				self.iter = Some(iter);
				continue;
			};
			let Ok(metadata) = entry.metadata() else {
				self.iter = Some(iter);
				continue;
			};
			if metadata.is_dir() {
				let Ok(entry_iter) = std::fs::read_dir(entry.path()) else {
					self.iter = Some(iter);
					continue;
				};
				self.stack.push(iter);
				self.iter = Some(entry_iter);
				continue;
			}
			if !metadata.is_file() {
				self.iter = Some(iter);
				continue;
			}
			self.iter = Some(iter);
			return Some(entry.path());
		}
	}
}
