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
		self.handle.dispatch(Action::UpdateProgress { id: self.id, value: self.value, max: self.max, new_name: None });
	}

	pub fn inc(&mut self, amt: u32) {
		self.value += amt;
		self.dispatch();
	}

	pub fn inc_max(&mut self, amt: u32) {
		self.max += amt;
		self.dispatch();
	}

	pub fn value(&self) -> u32 {
		self.value
	}

	pub fn max(&self) -> u32 {
		self.max
	}

	pub fn set_name(&mut self, name: String, value: u32, max: u32) {
		self.value = value;
		self.max = max;
		self.handle.dispatch(Action::UpdateProgress {
			id: self.id,
			new_name: Some(name),
			value: self.value,
			max: self.max,
		});
	}
}
