mod class_level;
pub use class_level::*;

pub trait Evaluator {
	type Item;
}

#[derive(Clone)]
pub struct BoxedEvaluator<V>(std::rc::Rc<dyn Evaluator<Item = V> + 'static>);
impl<V> PartialEq for BoxedEvaluator<V> {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.0, &other.0)
	}
}
impl<V> std::ops::Deref for BoxedEvaluator<V> {
	type Target = std::rc::Rc<dyn Evaluator<Item = V> + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T, V> From<T> for BoxedEvaluator<V>
where
	T: Evaluator<Item = V> + 'static,
{
	fn from(value: T) -> Self {
		Self(std::rc::Rc::new(value))
	}
}
