use std::ops::AddAssign;

mod error;
pub use error::*;
pub mod selector;
mod value;
pub use value::*;

mod trait_object_eq;
pub use trait_object_eq::*;
pub mod web_ext;
pub use web_ext::*;

pub type BoxAny = Box<dyn std::any::Any + 'static + Send + Sync>;

#[derive(thiserror::Error, Debug)]
#[error(
	"Incompatible {0} types: \
	the evaluator specified by kdl {1} has the {0} type {2}, \
	but the node is expecting an {0} type of {3}."
)]
pub struct IncompatibleTypes(pub &'static str, pub &'static str, pub &'static str, pub &'static str);

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

pub trait PathExt {
	fn normalize(&self) -> std::path::PathBuf;
}
impl PathExt for std::path::Path {
	/// Normalize a path, removing things like `.` and `..`.
	///
	/// Adapted from cargo
	/// https://github.com/rust-lang/cargo/blob/403fbe2b490d6cbb715ed768462bb7f977a6d514/crates/cargo-util/src/paths.rs#L84
	fn normalize(&self) -> std::path::PathBuf {
		use std::path::{Component, PathBuf};
		let mut components = self.components().peekable();
		let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
			components.next();
			PathBuf::from(c.as_os_str())
		} else {
			PathBuf::new()
		};

		for component in components {
			match component {
				Component::Prefix(..) => unreachable!(),
				Component::RootDir => {
					ret.push(component.as_os_str());
				}
				Component::CurDir => {}
				Component::ParentDir => {
					ret.pop();
				}
				Component::Normal(c) => {
					ret.push(c);
				}
			}
		}
		ret
	}
}
