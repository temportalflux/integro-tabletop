use super::character::Persistent;
use dyn_clone::{clone_trait_object, DynClone};

pub mod armor;

pub trait Criteria: DynClone {
	fn evaluate(&self, character: &Persistent) -> Result<(), String>;
}
clone_trait_object!(Criteria);

#[derive(Clone)]
pub struct BoxedCriteria(std::rc::Rc<dyn Criteria + 'static>);
impl PartialEq for BoxedCriteria {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for BoxedCriteria {
	type Target = std::rc::Rc<dyn Criteria + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T> From<T> for BoxedCriteria
where
	T: Criteria + 'static,
{
	fn from(value: T) -> Self {
		Self(std::rc::Rc::new(value))
	}
}
