use super::{Action, List};
use uuid::Uuid;
use yew::prelude::*;

#[derive(Clone)]
pub struct ProgressHandle {
	pub(super) handle: UseReducerHandle<List>,
	pub(super) id: Uuid,
	pub(super) value: u32,
	pub(super) max: u32,
}
impl ProgressHandle {
	fn dispatch(&self) {
		self.handle.dispatch(Action::UpdateProgress {
			id: self.id,
			value: self.value,
			max: self.max,
		});
	}

	pub fn inc(&mut self, amt: u32) {
		self.value += amt;
		self.dispatch();
	}

	pub fn value(&self) -> u32 {
		self.value
	}

	pub fn max(&self) -> u32 {
		self.max
	}
}
