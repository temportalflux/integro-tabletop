use crate::{
	database::{
		app::{Criteria, Database, Entry},
		Error as DatabaseError,
	},
	kdl_ext::{FromKDL, KDLNode},
	system::{self, core::SourceId, dnd5e::SystemComponent},
};
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
};
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
	async_handle: UseAsyncHandle<Vec<T>, DatabaseError>,
	run: Rc<dyn Fn(Option<QueryAllArgs<T>>)>,
}
pub enum QueryStatus<T> {
	Empty,
	Pending,
	Success(T),
	Failed(DatabaseError),
}

impl<T> UseQueryAllHandle<T> {
	pub fn status(&self) -> QueryStatus<&Vec<T>> {
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

	pub fn run(&self, args: Option<QueryAllArgs<T>>) {
		(self.run)(args);
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
				let Some(args) = &*guard else { return Ok(Vec::new()); };
				args.clone()
			};
			if system_id.is_empty() {
				return Ok(Vec::new());
			}
			let mut query = database
				.query_entries(system_id, category, criteria)
				.await?;
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
pub fn use_query_all_typed<T>(
	auto_fetch: bool,
	initial_args: Option<QueryAllArgs<T>>,
) -> UseQueryAllHandle<T>
where
	T: KDLNode + FromKDL + SystemComponent + Unpin + Clone + 'static,
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
				let Some(args) = &*guard else { return Ok(Vec::new()); };
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
pub fn use_typed_fetch_callback<Item>(
	task_name: String,
	fn_item: Callback<Item>,
) -> Callback<SourceId>
where
	Item: 'static + KDLNode + FromKDL + SystemComponent + Unpin,
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
			let Some(item) = database.get_typed_entry::<Item>(source_id, system_depot).await? else {
				return Ok(());
			};
			fn_item.emit(item);
			Ok(()) as Result<(), crate::database::app::FetchError>
		});
	})
}

#[hook]
pub fn use_typed_fetch_callback_tuple<Item, Arg>(
	task_name: String,
	fn_item: Callback<(Item, Arg)>,
) -> Callback<(SourceId, Arg)>
where
	Item: 'static + KDLNode + FromKDL + SystemComponent + Unpin,
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
			let Some(item) = database.get_typed_entry::<Item>(source_id.clone(), system_depot).await? else {
				log::error!("No such database entry {:?}", source_id.to_string());
				return Ok(());
			};
			fn_item.emit((item, arg));
			Ok(()) as Result<(), crate::database::app::FetchError>
		});
	})
}
