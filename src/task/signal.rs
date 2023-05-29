use std::{
	rc::Rc,
	sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::Notify;

/// Semaphore-like flag that can be awaited on.
/// Based on [future_bool](https://github.com/devalain/future-bool), but for wasm.
#[derive(Clone)]
pub struct Signal {
	inner: Rc<AtomicBool>,
	notify_true: Rc<Notify>,
	notify_false: Rc<Notify>,
}

impl Signal {
	pub fn new(value: bool) -> Self {
		Self {
			inner: Rc::new(AtomicBool::new(value)),
			notify_true: Rc::new(Notify::new()),
			notify_false: Rc::new(Notify::new()),
		}
	}

	/// Sets the `bool` value to `true`.
	pub fn set(&self) {
		self.inner.store(true, Ordering::Release);
		self.notify_true.notify_waiters();
	}

	/// Sets the `bool` value to `false`.
	pub fn unset(&self) {
		self.inner.store(false, Ordering::Release);
		self.notify_false.notify_waiters();
	}

	/// Returns the new value when it has changed.
	pub async fn wait_change(&self) -> bool {
		let val = self.inner.load(Ordering::Acquire);
		if val {
			self.notify_false.notified().await;
		} else {
			self.notify_true.notified().await;
		}
		!val
	}

	/// If the value is `true`, returns immidiately, otherwise waits until it's `true`.
	pub async fn wait_true(&self) {
		let val = self.inner.load(Ordering::Acquire);
		if !val {
			self.notify_true.notified().await;
		}
	}

	/// If the value is `false`, returns immidiately, otherwise waits until it's `false`.
	pub async fn wait_false(&self) {
		let val = self.inner.load(Ordering::Acquire);
		if val {
			self.notify_false.notified().await;
		}
	}
}
