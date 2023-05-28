use graphql_client::GraphQLQuery;

mod stream;
pub use stream::*;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
	#[error("request error")]
	ReqwestError(#[from] reqwest::Error),
	#[error("No data in response")]
	NoData,
}
pub type QueryResult<Query> = Result<<Query as GraphQLQuery>::ResponseData, QueryError>;
pub type QueryFuture<Query> = futures_util::future::LocalBoxFuture<'static, QueryResult<Query>>;

static GITHUB_API_GRAPHQL: &'static str = "https://api.github.com/graphql";

pub trait GraphQLQueryExt<T: GraphQLQuery + 'static> {
	fn post(client: reqwest::Client, vars: T::Variables) -> QueryFuture<T>;
}
impl<T> GraphQLQueryExt<T> for T
where
	T: GraphQLQuery + 'static,
	T::Variables: Send + Sync,
{
	fn post(client: reqwest::Client, vars: T::Variables) -> QueryFuture<T> {
		use graphql_client::reqwest::post_graphql;
		Box::pin(async move {
			let client = client;
			let inner = post_graphql::<T, _>(&client, GITHUB_API_GRAPHQL, vars);
			let response = match inner.await {
				Ok(response) => response,
				Err(err) => return Err(QueryError::ReqwestError(err)),
			};
			let data = match response.data {
				Some(data) => data,
				None => return Err(QueryError::NoData),
			};
			Ok(data)
		})
	}
}
