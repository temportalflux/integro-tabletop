mod evaluator;
use std::ops::AddAssign;

pub use evaluator::*;
mod error;
pub use error::*;
mod mutator;
pub use mutator::*;
pub mod selector;
mod value;
pub use value::*;

mod trait_object_eq;
pub use trait_object_eq::*;
pub mod web_ext;
pub use web_ext::*;

pub type PinFuture<T> = PinFutureLifetime<'static, T>;
pub type PinFutureLifetime<'l, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'l + Send>>;
pub type PinFutureLifetimeNoSend<'l, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'l>>;

pub fn list_as_english(mut items: Vec<String>, joiner: &str) -> Option<String> {
	match items.len() {
		0 => None,
		1 => Some(items.into_iter().next().unwrap()),
		2 => Some(items.join(format!(" {joiner} ").as_str())),
		_ => {
			if let Some(last) = items.last_mut() {
				*last = format!("{joiner} {last}");
			}
			Some(items.join(", "))
		}
	}
}

pub trait AddAssignMap {
	fn add_assign_map(&mut self, other: &Self);
}
impl AddAssignMap for usize {
	fn add_assign_map(&mut self, other: &Self) {
		self.add_assign(other);
	}
}
impl<K, V> AddAssignMap for std::collections::BTreeMap<K, V>
where
	K: Clone + std::hash::Hash + std::cmp::Ord,
	V: Clone + AddAssignMap,
{
	fn add_assign_map(&mut self, other: &Self) {
		for (key, value) in other {
			match self.get_mut(key) {
				None => {
					self.insert(key.clone(), value.clone());
				}
				Some(dst_value) => {
					dst_value.add_assign_map(value);
				}
			}
		}
	}
}
