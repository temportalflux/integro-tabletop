use crate::{
	database::{
		app::{Criteria, Database},
		Error as DatabaseError,
	},
	kdl_ext::{FromKDL, KDLNode},
	system::{self, core::SourceId, dnd5e::SystemComponent},
};
use std::sync::Arc;
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncHandle, UseAsyncOptions};

pub struct QueryAllArgs<T> {
	pub auto_fetch: bool,
	pub system: String,
	pub criteria: Option<Box<Criteria>>,
	pub adjust_listings: Option<Arc<dyn Fn(Vec<T>) -> Vec<T> + 'static>>,
}
impl<T> Default for QueryAllArgs<T> {
	fn default() -> Self {
		Self {
			auto_fetch: false,
			system: Default::default(),
			criteria: None,
			adjust_listings: None,
		}
	}
}

pub struct UseQueryHandle<T> {
	async_handle: UseAsyncHandle<T, DatabaseError>,
}
pub enum QueryStatus<T> {
	Empty,
	Pending,
	Success(T),
	Failed(DatabaseError),
}

impl<T> UseQueryHandle<T> {
	pub fn status(&self) -> QueryStatus<&T> {
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
		self.async_handle.run();
	}
}

#[hook]
pub fn use_query_all_typed<T>(args: QueryAllArgs<T>) -> UseQueryHandle<Vec<T>>
where
	T: KDLNode + FromKDL + SystemComponent + Unpin + Clone + 'static,
{
	let QueryAllArgs {
		auto_fetch,
		system: system_id,
		criteria,
		adjust_listings,
	} = args;
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let async_handle = use_async_with_options(
		async move {
			let query = database.query_typed::<T>(system_id, system_depot, criteria);
			let mut typed_entries = query.await?.all().await;
			if let Some(adjust_listings) = &adjust_listings {
				typed_entries = (adjust_listings)(typed_entries);
			}
			Ok(typed_entries)
		},
		UseAsyncOptions { auto: auto_fetch },
	);
	UseQueryHandle { async_handle }
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
			Ok(())
		});
	})
}
