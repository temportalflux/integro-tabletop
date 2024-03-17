mod criteria;
pub use criteria::*;
mod old;
pub use old::*;
mod source;
pub use source::*;
mod parse;
pub use parse::*;

/// Sources of data that exist within a database and can be called to fetch the underlying data.
pub trait QuerySource {
	type Output;
	#[allow(async_fn_in_trait)]
	async fn execute(self, database: &crate::database::Database) -> Self::Output;
}
