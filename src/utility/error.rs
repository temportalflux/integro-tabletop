use enumset::{EnumSet, EnumSetType};

#[derive(thiserror::Error, Debug)]
#[error("Invalid value {0:?}, expected one of: {1:?}")]
pub struct NotInList(pub String, pub Vec<&'static str>);

#[derive(thiserror::Error, Debug)]
#[error("Invalid valid {0:?}, expected one of: {1:?}")]
pub struct InvalidEnumStr<T: EnumSetType + ToString>(String, EnumSet<T>);
impl<S, T> From<S> for InvalidEnumStr<T>
where
	T: EnumSetType + ToString,
	S: Into<String>,
{
	fn from(value: S) -> Self {
		Self(value.into(), EnumSet::all())
	}
}
