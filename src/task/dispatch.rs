use super::{Action, List, ProgressHandle, Signal};
use futures_util::Future;
use std::rc::Rc;
use uuid::Uuid;
use yew::prelude::*;

#[derive(Clone)]
pub struct Dispatch(pub(super) Rc<UseReducerHandle<List>>);
impl PartialEq for Dispatch {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}
impl Dispatch {
	pub fn new_progress(&self, max: u32) -> ProgressHandle {
		ProgressHandle {
			id: Uuid::new_v4(),
			handle: (*self.0).clone(),
			value: 0,
			max,
		}
	}

	pub fn spawn<F>(
		&self,
		name: impl Into<String>,
		progress: Option<ProgressHandle>,
		pending: F,
	) -> Signal
	where
		F: Future<Output = anyhow::Result<()>> + 'static,
	{
		let signal = Signal::new(false);
		let pending = Box::pin({
			let signal = signal.clone();
			async move {
				pending.await?;
				signal.set();
				Ok(())
			}
		});
		self.0.dispatch(Action::Insert {
			handle: (*self.0).clone(),
			name: name.into(),
			progress,
			pending,
		});
		signal
	}
}
