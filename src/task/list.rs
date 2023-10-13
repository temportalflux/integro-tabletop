use super::ProgressHandle;
use futures_util::future::LocalBoxFuture;
use std::{collections::HashMap, rc::Rc};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub struct List {
	pub(super) display_order: Vec<Uuid>,
	pub(super) tasks: HashMap<Uuid, Handle>,
}

#[derive(Clone, PartialEq)]
pub struct Handle {
	pub name: AttrValue,
	pub status: Status,
	pub progress: Option<(u32, u32)>,
}

#[derive(Clone)]
pub enum Status {
	Pending,
	Failed(Rc<anyhow::Error>),
}
impl PartialEq for Status {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Pending, Self::Pending) => true,
			(Self::Failed(_), Self::Failed(_)) => true,
			_ => false,
		}
	}
}

impl List {
	pub fn insert(
		&mut self,
		handle: UseReducerHandle<Self>,
		name: String,
		progress: Option<ProgressHandle>,
		pending: LocalBoxFuture<'static, anyhow::Result<()>>,
	) {
		let id = progress
			.as_ref()
			.map(|progress| progress.id)
			.unwrap_or_else(|| Uuid::new_v4());

		let idx = self.get_insertion_idx(&name);
		self.display_order.insert(idx, id);
		self.tasks.insert(
			id,
			Handle {
				name: name.into(),
				status: Status::Pending,
				progress: progress.map(|progress| (progress.value(), progress.max())),
			},
		);

		// Spawn the task, with logic to remove it from the task list when it is complete.
		spawn_local(Box::pin(async move {
			let result = pending.await;
			match result {
				Ok(()) => {
					handle.dispatch(Action::Remove { id });
				}
				Err(error) => {
					// Failed tasks are marked as failed
					handle.dispatch(Action::MarkFailed { id, error });
					// and then removed after a 30s delay
					spawn_local(Box::pin(async move {
						gloo_timers::future::TimeoutFuture::new(100 * 30).await;
						handle.dispatch(Action::Remove { id });
					}));
				}
			}
		}));
	}

	fn get_insertion_idx(&self, a_name: &String) -> usize {
		use std::cmp::Ordering;
		let idx = self.display_order.binary_search_by(|b_id| {
			let Some(b_task) = self.tasks.get(b_id) else {
				return Ordering::Less;
			};
			a_name.as_str().cmp(b_task.name.as_str())
		});
		idx.unwrap_or_else(|err_idx| err_idx)
	}

	fn set_progress(&mut self, id: &Uuid, value: u32, max: u32, new_name: Option<String>) {
		let handle = self.tasks.get_mut(id).expect("task handle went missing");
		if let Some(name) = new_name {
			handle.name = name.into();
		}
		if let Some(progress) = &mut handle.progress {
			progress.0 = value;
			progress.1 = max;
		}
	}

	fn remove(&mut self, id: &Uuid) {
		self.display_order.retain(|task_id| task_id != id);
		self.tasks.remove(id);
	}

	fn mark_failed(&mut self, id: &Uuid, error: anyhow::Error) {
		let handle = self.tasks.get_mut(id).expect("task handle went missing");
		log::error!("Task {:?} failed: {error:?}", handle.name);
		handle.status = Status::Failed(Rc::new(error));
	}
}

pub enum Action {
	Insert {
		handle: UseReducerHandle<List>,
		name: String,
		progress: Option<ProgressHandle>,
		pending: LocalBoxFuture<'static, anyhow::Result<()>>,
	},
	UpdateProgress {
		id: Uuid,
		value: u32,
		max: u32,
		new_name: Option<String>,
	},
	Remove {
		id: Uuid,
	},
	MarkFailed {
		id: Uuid,
		error: anyhow::Error,
	},
}

impl yew::Reducible for List {
	type Action = Action;

	fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		let list = Rc::make_mut(&mut self);
		match action {
			Action::Insert {
				handle,
				name,
				progress,
				pending,
			} => {
				list.insert(handle, name, progress, pending);
			}
			Action::UpdateProgress {
				id,
				value,
				max,
				new_name,
			} => {
				list.set_progress(&id, value, max, new_name);
			}
			Action::Remove { id } => {
				list.remove(&id);
			}
			Action::MarkFailed { id, error } => {
				list.mark_failed(&id, error);
			}
		}
		self
	}
}
