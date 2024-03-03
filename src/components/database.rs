use crate::{
	database::{Criteria, Database, Entry, FetchError, Module},
	system::{self, core::SourceId, dnd5e::SystemBlock},
};
use std::{
	collections::BTreeMap,
	rc::Rc,
	sync::{Arc, Mutex},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncHandle, UseAsyncOptions};

#[derive(Clone)]
pub struct QueryAllArgs<T> {
	pub system: String,
	pub criteria: Option<Box<Criteria>>,
	pub adjust_listings: Option<Arc<dyn Fn(Vec<T>) -> Vec<T> + 'static>>,
	pub max_limit: Option<usize>,
}
impl<T> Default for QueryAllArgs<T> {
	fn default() -> Self {
		Self {
			system: Default::default(),
			criteria: None,
			adjust_listings: None,
			max_limit: None,
		}
	}
}

#[derive(Clone)]
pub struct UseQueryAllHandle<T> {
	async_handle: UseAsyncHandle<Vec<T>, database::Error>,
	run: Rc<dyn Fn(Option<QueryAllArgs<T>>)>,
}

impl<T: PartialEq> PartialEq for UseQueryAllHandle<T> {
	fn eq(&self, other: &Self) -> bool {
		self.async_handle == other.async_handle
	}
}

#[derive(Debug, PartialEq)]
pub enum QueryStatus<T, E> {
	Empty,
	Pending,
	Success(T),
	Failed(E),
}

impl<T> UseQueryAllHandle<T> {
	pub fn status(&self) -> QueryStatus<&Vec<T>, database::Error> {
		if self.async_handle.loading {
			return QueryStatus::Pending;
		}
		if let Some(error) = &self.async_handle.error {
			return QueryStatus::Failed(error.clone());
		}
		if let Some(data) = &self.async_handle.data {
			return QueryStatus::Success(data);
		}
		QueryStatus::Empty
	}

	pub fn get(&self, idx: usize) -> Option<&T> {
		let Some(data) = &self.async_handle.data else {
			return None;
		};
		data.get(idx)
	}

	pub fn run(&self, args: Option<QueryAllArgs<T>>) {
		(self.run)(args);
	}
}

#[derive(Clone)]
pub struct UseQueryModulesHandle {
	async_handle: UseAsyncHandle<Vec<Module>, database::Error>,
	run: Rc<dyn Fn()>,
}

impl PartialEq for UseQueryModulesHandle {
	fn eq(&self, other: &Self) -> bool {
		self.async_handle == other.async_handle
	}
}

impl UseQueryModulesHandle {
	pub fn status(&self) -> QueryStatus<&Vec<Module>, database::Error> {
		if self.async_handle.loading {
			return QueryStatus::Pending;
		}
		if let Some(error) = &self.async_handle.error {
			return QueryStatus::Failed(error.clone());
		}
		if let Some(data) = &self.async_handle.data {
			return QueryStatus::Success(data);
		}
		QueryStatus::Empty
	}

	pub fn run(&self) {
		(self.run)();
	}
}

#[hook]
pub fn use_query_all(
	category: impl Into<String>,
	auto_fetch: bool,
	initial_args: Option<QueryAllArgs<Entry>>,
) -> UseQueryAllHandle<Entry> {
	let database = use_context::<Database>().unwrap();
	let options = UseAsyncOptions { auto: auto_fetch };
	let args_handle = Rc::new(Mutex::new(initial_args));
	let async_args = args_handle.clone();
	let category = category.into();
	let async_handle = use_async_with_options(
		async move {
			use futures_util::stream::StreamExt;
			let QueryAllArgs {
				system: system_id,
				criteria,
				adjust_listings,
				max_limit,
			} = {
				let guard = async_args.lock().unwrap();
				let Some(args) = &*guard else {
					return Ok(Vec::new());
				};
				args.clone()
			};
			if system_id.is_empty() {
				return Ok(Vec::new());
			}
			let mut query = database.query_entries(system_id, category, criteria).await?;
			let mut items = Vec::new();
			while let Some(item) = query.next().await {
				items.push(item);
				if let Some(limit) = &max_limit {
					if items.len() >= *limit {
						break;
					}
				}
			}
			if let Some(adjust_listings) = &adjust_listings {
				items = (adjust_listings)(items);
			}
			Ok(items)
		},
		options,
	);
	let run = Rc::new({
		let handle = async_handle.clone();
		move |args: Option<QueryAllArgs<Entry>>| {
			*args_handle.lock().unwrap() = args;
			handle.run();
		}
	});
	UseQueryAllHandle { async_handle, run }
}

#[hook]
pub fn use_query_all_typed<T>(auto_fetch: bool, initial_args: Option<QueryAllArgs<T>>) -> UseQueryAllHandle<T>
where
	T: SystemBlock + Unpin + Clone + 'static,
{
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let options = UseAsyncOptions { auto: auto_fetch };
	let args_handle = Rc::new(Mutex::new(initial_args));
	let async_args = args_handle.clone();
	let async_handle = use_async_with_options(
		async move {
			let QueryAllArgs {
				system: system_id,
				criteria,
				adjust_listings,
				max_limit,
			} = {
				let guard = async_args.lock().unwrap();
				let Some(args) = &*guard else {
					return Ok(Vec::new());
				};
				args.clone()
			};
			if system_id.is_empty() {
				return Ok(Vec::new());
			}
			let query = database.query_typed::<T>(system_id, system_depot, criteria);
			let mut typed_entries = query.await?.first_n(max_limit).await;
			if let Some(adjust_listings) = &adjust_listings {
				typed_entries = (adjust_listings)(typed_entries);
			}
			Ok(typed_entries)
		},
		options,
	);
	let run = Rc::new({
		let handle = async_handle.clone();
		move |args: Option<QueryAllArgs<T>>| {
			if let Some(args) = args {
				*args_handle.lock().unwrap() = Some(args);
			}
			handle.run();
		}
	});
	UseQueryAllHandle { async_handle, run }
}

#[hook]
pub fn use_query_modules(system: Option<&'static str>) -> UseQueryModulesHandle {
	let database = use_context::<Database>().unwrap();
	let options = UseAsyncOptions { auto: true };
	let system = system.map(|s| std::borrow::Cow::Borrowed(s));
	let async_handle = use_async_with_options(
		async move {
			let items = database.query_modules(system).await?;
			Ok(items)
		},
		options,
	);
	let run = Rc::new({
		let handle = async_handle.clone();
		move || {
			handle.run();
		}
	});
	UseQueryModulesHandle { async_handle, run }
}

#[derive(Clone)]
pub struct UseQueryDiscreteHandle<T, E> {
	state: UseStateHandle<QueryStatus<(Vec<SourceId>, BTreeMap<SourceId, T>), E>>,
	run: Rc<dyn Fn(Vec<SourceId>)>,
}
impl<T, E> PartialEq for UseQueryDiscreteHandle<T, E>
where
	T: PartialEq,
	E: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		*self.state == *other.state
	}
}

impl<T, E> UseQueryDiscreteHandle<T, E> {
	pub fn status(&self) -> &QueryStatus<(Vec<SourceId>, BTreeMap<SourceId, T>), E> {
		&*self.state
	}

	pub fn clear(&self) {
		self.state.set(QueryStatus::Empty);
	}

	pub fn run(&self, args: Vec<SourceId>) {
		(self.run)(args);
	}
}

type QueryEntriesStatus = QueryStatus<(Vec<SourceId>, BTreeMap<SourceId, Entry>), database::Error>;
#[hook]
pub fn use_query_entries() -> UseQueryDiscreteHandle<Entry, database::Error> {
	let database = use_context::<Database>().unwrap();
	let status = use_state(|| QueryEntriesStatus::Empty);
	let run = Rc::new({
		let status = status.clone();
		move |args: Vec<SourceId>| {
			if args.is_empty() {
				status.set(QueryStatus::Empty);
				return;
			}

			status.set(QueryStatus::Pending);

			let database = database.clone();
			let perform_query = async move {
				let mut items = BTreeMap::new();
				for id in &args {
					let Some(item) = database.get::<Entry>(id.to_string()).await? else {
						continue;
					};
					items.insert(id.clone(), item);
				}
				Ok((args, items))
			};

			let status = status.clone();
			spawn_local(async move {
				status.set(match perform_query.await {
					Err(err) => QueryStatus::Failed(err),
					Ok((ids, data)) => {
						if data.is_empty() {
							QueryStatus::Empty
						} else {
							QueryStatus::Success((ids, data))
						}
					}
				});
			});
		}
	});
	UseQueryDiscreteHandle { run, state: status }
}

type QueryTypedStatus<T> = QueryStatus<(Vec<SourceId>, BTreeMap<SourceId, T>), FetchError>;
#[hook]
pub fn use_query_typed<T>() -> UseQueryDiscreteHandle<T, FetchError>
where
	T: SystemBlock + Unpin + Clone + 'static,
{
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let status = use_state(|| QueryTypedStatus::<T>::Empty);
	let run = Rc::new({
		let status = status.clone();
		move |new_ids: Vec<SourceId>| {
			if matches!(*status, QueryStatus::Pending) {
				return;
			}
			if new_ids.is_empty() {
				status.set(QueryStatus::Empty);
				return;
			}
			if let QueryStatus::Success((prev_ids, prev_items)) = &*status {
				if new_ids == *prev_ids {
					let mut contains_all_ids = true;
					for id in &new_ids {
						if !prev_items.contains_key(id) {
							contains_all_ids = false;
							break;
						}
					}
					if contains_all_ids {
						return;
					}
				}
			}

			status.set(QueryStatus::Pending);

			let perform_query = {
				let database = database.clone();
				let system_depot = system_depot.clone();
				async move {
					let mut items = BTreeMap::new();
					for id in &new_ids {
						let Some(item) = database
							.get_typed_entry::<T>(id.clone(), system_depot.clone(), None)
							.await?
						else {
							continue;
						};
						items.insert(id.clone(), item);
					}
					Ok((new_ids, items))
				}
			};

			let status = status.clone();
			spawn_local(async move {
				status.set(match perform_query.await {
					Err(err) => QueryStatus::Failed(err),
					Ok((ids, items)) => {
						if items.is_empty() {
							QueryStatus::Empty
						} else {
							QueryStatus::Success((ids, items))
						}
					}
				});
			});
		}
	});
	UseQueryDiscreteHandle { run, state: status }
}

#[hook]
pub fn use_typed_fetch_callback<EntryContent>(task_name: String, fn_item: Callback<EntryContent>) -> Callback<SourceId>
where
	EntryContent: 'static + SystemBlock + Unpin,
	Event: 'static,
{
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let task_dispatch = use_context::<crate::task::Dispatch>().unwrap();
	Callback::from(move |source_id: SourceId| {
		let database = database.clone();
		let system_depot = system_depot.clone();
		let fn_item = fn_item.clone();
		task_dispatch.spawn(task_name.clone(), None, async move {
			let Some(item) = database
				.get_typed_entry::<EntryContent>(source_id.clone(), system_depot, None)
				.await?
			else {
				log::error!(target: "database", "Failed to find entry for {:?}", source_id.to_string());
				return Ok(());
			};
			fn_item.emit(item);
			Ok(()) as Result<(), FetchError>
		});
	})
}

#[hook]
pub fn use_typed_fetch_callback_tuple<Item, Arg>(
	task_name: String,
	fn_item: Callback<(Item, Arg)>,
) -> Callback<(SourceId, Arg)>
where
	Item: 'static + SystemBlock + Unpin,
	Event: 'static,
	Arg: 'static,
{
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let task_dispatch = use_context::<crate::task::Dispatch>().unwrap();
	Callback::from(move |(source_id, arg): (SourceId, Arg)| {
		let database = database.clone();
		let system_depot = system_depot.clone();
		let fn_item = fn_item.clone();
		task_dispatch.spawn(task_name.clone(), None, async move {
			let Some(item) = database
				.get_typed_entry::<Item>(source_id.clone(), system_depot, None)
				.await?
			else {
				log::error!("No such database entry {:?}", source_id.to_string());
				return Ok(());
			};
			fn_item.emit((item, arg));
			Ok(()) as Result<(), FetchError>
		});
	})
}
