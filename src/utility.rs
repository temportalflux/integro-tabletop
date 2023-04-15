mod evaluator;
pub use evaluator::*;
mod error;
pub use error::*;
mod mutator;
pub use mutator::*;
mod selector;
pub use selector::*;
mod value;
pub use value::*;

mod trait_object_eq;
pub use trait_object_eq::*;

pub type PinFuture<T> = PinFutureLifetime<'static, T>;
pub type PinFutureLifetime<'l, T> =
	std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'l + Send>>;
pub type PinFutureLifetimeNoSend<'l, T> =
	std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'l>>;

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
