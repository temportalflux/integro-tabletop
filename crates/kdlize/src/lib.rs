
mod builder;
pub use builder::*;

pub mod error;
pub mod ext;

mod reader;
pub use reader::*;

pub trait NodeId {
	fn id() -> &'static str
	where
		Self: Sized;

	fn get_id(&self) -> &'static str;
}

#[macro_export]
macro_rules! impl_kdl_node {
	($target:ty, $id:expr) => {
		impl kdlize::NodeId for $target {
			fn id() -> &'static str {
				$id
			}

			fn get_id(&self) -> &'static str {
				$id
			}
		}
	};
}

pub trait AsKdl {
	fn as_kdl(&self) -> NodeBuilder;
}
macro_rules! impl_askdl_entry {
	($target:ty, $map:expr) => {
		impl AsKdl for $target {
			fn as_kdl(&self) -> NodeBuilder {
				NodeBuilder::default().with_entry(($map)(*self))
			}
		}
	};
}
impl_askdl_entry!(bool, |v| v);
impl_askdl_entry!(u8, |v| v as i64);
impl_askdl_entry!(i8, |v| v as i64);
impl_askdl_entry!(u16, |v| v as i64);
impl_askdl_entry!(i16, |v| v as i64);
impl_askdl_entry!(u32, |v| v as i64);
impl_askdl_entry!(i32, |v| v as i64);
impl_askdl_entry!(u64, |v| v as i64);
impl_askdl_entry!(i64, |v| v);
impl_askdl_entry!(u128, |v| v as i64);
impl_askdl_entry!(i128, |v| v as i64);
impl_askdl_entry!(usize, |v| v as i64);
impl_askdl_entry!(isize, |v| v as i64);
impl_askdl_entry!(f32, |v| v as f64);
impl_askdl_entry!(f64, |v| v);
impl AsKdl for String {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if !self.is_empty() {
			node.push_entry(self.clone());
		}
		node
	}
}

pub trait FromKdl<Context> {
	type Error;

	fn from_kdl<'doc>(node: &mut NodeReader<'doc, Context>) -> Result<Self, Self::Error>
	where
		Self: Sized;
}

macro_rules! impl_fromkdl {
	($target:ty, $method:ident, $map:expr) => {
		impl<Context> FromKdl<Context> for $target {
			type Error = crate::error::Error;
			fn from_kdl<'doc>(node: &mut NodeReader<'doc, Context>) -> Result<Self, Self::Error> {
				Ok(node.$method().map($map)?)
			}
		}
	};
}
impl_fromkdl!(bool, next_bool_req, |v| v);
impl_fromkdl!(u8, next_i64_req, |v| v as u8);
impl_fromkdl!(i8, next_i64_req, |v| v as i8);
impl_fromkdl!(u16, next_i64_req, |v| v as u16);
impl_fromkdl!(i16, next_i64_req, |v| v as i16);
impl_fromkdl!(u32, next_i64_req, |v| v as u32);
impl_fromkdl!(i32, next_i64_req, |v| v as i32);
impl_fromkdl!(u64, next_i64_req, |v| v as u64);
impl_fromkdl!(i64, next_i64_req, |v| v);
impl_fromkdl!(u128, next_i64_req, |v| v as u128);
impl_fromkdl!(i128, next_i64_req, |v| v as i128);
impl_fromkdl!(usize, next_i64_req, |v| v as usize);
impl_fromkdl!(isize, next_i64_req, |v| v as isize);
impl_fromkdl!(f32, next_f64_req, |v| v as f32);
impl_fromkdl!(f64, next_f64_req, |v| v);
impl<Context> FromKdl<Context> for String {
	type Error = crate::error::Error;
	fn from_kdl<'doc>(node: &mut NodeReader<'doc, Context>) -> Result<Self, Self::Error> {
		Ok(node.next_str_req()?.to_string())
	}
}

impl<T, Context> FromKdl<Context> for Option<T>
where
	T: FromKdl<Context>,
{
	type Error = T::Error;
	fn from_kdl<'doc>(node: &mut NodeReader<'doc, Context>) -> Result<Self, Self::Error> {
		// Instead of consuming the next-idx, just peek to see if there is a value there or not.
		match node.peak_opt() {
			Some(_) => T::from_kdl(node).map(|v| Some(v)),
			None => Ok(None),
		}
	}
}
