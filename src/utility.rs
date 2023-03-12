mod evaluator;
pub use evaluator::*;
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
