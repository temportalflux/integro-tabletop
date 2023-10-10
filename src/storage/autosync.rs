use crate::{
	database::app::Module,
	system::core::{ModuleId, SourceId},
};
use std::{collections::BTreeMap, rc::Rc};
use yew::{html::ChildrenProps, prelude::*};
use yew_hooks::*;

#[derive(Clone)]
pub struct Channel(Rc<RequestChannel>);
impl PartialEq for Channel {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for Channel {
	type Target = RequestChannel;

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

pub struct RequestChannel {
	send_req: async_channel::Sender<Request>,
	recv_req: async_channel::Receiver<Request>,
}
impl RequestChannel {
	pub fn try_send_req(&self, req: Request) {
		let _ = self.send_req.try_send(req);
	}
}

#[derive(Debug)]
pub enum Request {
	// Only poll for what the latest version is of all installed modules.
	// This should not actually download any updates.
	FetchLatestVersionAllModules,
	// Polls what the latest version is for each provided module,
	// queuing downloads for each module which is not the latest version.
	FetchAndUpdateModules(Vec<ModuleId>),
	// Poll what the latest version is for this specific source file.
	// If there is an update, download the updates.
	UpdateFile(SourceId),
}

#[derive(thiserror::Error, Debug, Clone)]
enum StorageSyncError {
	#[error(transparent)]
	Database(#[from] crate::database::Error),
}

#[function_component]
pub fn Provider(props: &ChildrenProps) -> Html {
	let database = use_context::<crate::database::app::Database>().unwrap();
	let channel = Channel(use_memo(
		|_| {
			let (send_req, recv_req) = async_channel::unbounded();
			RequestChannel { send_req, recv_req }
		},
		(),
	));
	use_async_with_options(
		{
			let database = database.clone();
			let recv_req = channel.recv_req.clone();
			async move {
				while let Ok(req) = recv_req.recv().await {
					let mut modules = BTreeMap::new();
					let mut scan_for_new_modules = false;
					let mut update_modules_out_of_date = false;
					match req {
						Request::FetchLatestVersionAllModules => {
							scan_for_new_modules = true;
							for module in database.clone().query_modules(None).await? {
								modules.insert(module.id.clone(), module);
							}
						}
						Request::FetchAndUpdateModules(module_ids) => {
							update_modules_out_of_date = true;
							for id in module_ids {
								let module = database.get::<Module>(id.to_string()).await?;
								if let Some(module) = module {
									modules.insert(module.id.clone(), module);
								}
							}
						}
						Request::UpdateFile(source_id) => {
							update_modules_out_of_date = true;
							if let Some(id) = source_id.module {
								let module = database.get::<Module>(id.to_string()).await?;
								if let Some(module) = module {
									modules.insert(module.id.clone(), module);
								}
							}
						}
					}

					if scan_for_new_modules {
						// scan user for all integro modules that could be installed
						log::debug!(target: "autosync", "scan for new modules");
					}

					// fetch latest version of each module (save remote version to ddb)
					log::debug!(target: "autosync", "fetch latest versions for: {:?}", modules.keys().collect::<Vec<_>>());

					if update_modules_out_of_date {
						// scan modules for new content and download
						log::debug!(target: "autosync", "update out of date modules");
					}
				}
				Ok(()) as Result<(), StorageSyncError>
			}
		},
		UseAsyncOptions::enable_auto(),
	);

	html! {
		<ContextProvider<Channel> context={channel}>
			{props.children.clone()}
		</ContextProvider<Channel>>
	}
}
