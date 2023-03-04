use downcast_rs::{impl_downcast, Downcast};

pub trait TraitEq: Downcast {
	fn equals_trait(&self, other: &dyn TraitEq) -> bool;
}
impl_downcast!(TraitEq);

impl PartialEq for dyn TraitEq {
	fn eq(&self, other: &Self) -> bool {
		self.equals_trait(other)
	}
}

pub fn downcast_trait_eq<T>(a: &T, b: &dyn TraitEq) -> bool
where
	T: Downcast + PartialEq,
{
	b.as_any().downcast_ref::<T>().map_or(false, |b| a == b)
}

pub trait AsTraitEq<Super: ?Sized> {
	fn as_trait_eq(&self) -> &Super;
}
impl<'a, T: 'a + TraitEq> AsTraitEq<dyn TraitEq + 'a> for T {
	fn as_trait_eq(&self) -> &(dyn TraitEq + 'a) {
		self
	}
}
