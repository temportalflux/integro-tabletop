use crate::{
	database::Entry,
	system::{generics, Block},
};
use std::sync::Arc;

/// Parses the contents of a database `Entry` into a `Block` type.
pub struct ParseKdl<BlockType: Block> {
	generics: Arc<generics::Registry>,
	marker: std::marker::PhantomData<BlockType>,
}

impl<BlockType: Block> ParseKdl<BlockType> {
	pub fn new(generics: Arc<generics::Registry>) -> Self {
		Self {
			generics,
			marker: Default::default(),
		}
	}

	pub fn into_fn(self) -> impl FnMut(Entry) -> futures::future::Ready<Option<(Entry, BlockType)>> + 'static {
		move |entry: Entry| {
			futures::future::ready(match entry.parse_kdl::<BlockType>(self.generics.clone()) {
				None => None,
				Some(block) => Some((entry, block)),
			})
		}
	}

	pub fn new_fn(
		generics: Arc<generics::Registry>,
	) -> impl FnMut(Entry) -> futures::future::Ready<Option<(Entry, BlockType)>> + 'static {
		Self::new(generics).into_fn()
	}
}

pub trait DatabaseEntryStreamExt {
	type Input;
	fn parse_as<Output: 'static + Block>(
		self,
		generics: Arc<generics::Registry>,
	) -> futures::stream::LocalBoxStream<'static, (Self::Input, Output)>;
}
impl<T> DatabaseEntryStreamExt for T
where
	T: 'static + futures::Stream<Item = Entry>,
{
	type Input = Entry;
	fn parse_as<Output: 'static + Block>(
		self,
		generics: Arc<generics::Registry>,
	) -> futures::stream::LocalBoxStream<'static, (Self::Input, Output)> {
		use futures::StreamExt;
		self.filter_map(ParseKdl::<Output>::new_fn(generics)).boxed_local()
	}
}
