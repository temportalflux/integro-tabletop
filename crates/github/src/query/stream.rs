use super::{GraphQLQueryExt, QueryFuture};
use futures_util::FutureExt;
use graphql_client::GraphQLQuery;
use std::{pin::Pin, task::Poll};

#[derive(Debug)]
pub struct Cursor {
	pub has_next_page: bool,
	pub cursor: Option<String>,
}

pub trait StreamableQuery<T: GraphQLQuery> {
	type Item;
	fn update_variables(vars: &T::Variables, cursor: Option<String>) -> T::Variables;
	fn next(data: T::ResponseData) -> (Cursor, Self::Item);
}

pub struct QueryStream<Query: GraphQLQuery + StreamableQuery<Query>> {
	client: reqwest::Client,
	cursor: Cursor,
	variables: Query::Variables,
	active_query: Option<QueryFuture<Query>>,
}
impl<Query> QueryStream<Query>
where
	Query: GraphQLQuery + StreamableQuery<Query> + 'static,
	Query::Variables: Send + Sync + Unpin,
{
	pub fn new(client: reqwest::Client, variables: Query::Variables) -> Self {
		Self {
			client,
			cursor: Cursor {
				has_next_page: true,
				cursor: None,
			},
			variables,
			active_query: None,
		}
	}
}
impl<Query> futures_util::Stream for QueryStream<Query>
where
	Query: GraphQLQuery + StreamableQuery<Query> + 'static,
	Query::Variables: Send + Sync + Unpin,
{
	type Item = Query::Item;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
		if let Some(mut query) = self.active_query.take() {
			let Poll::Ready(result) = query.poll_unpin(cx) else {
				self.active_query = Some(query);
				return Poll::Pending;
			};
			let data = match result {
				Ok(data) => data,
				Err(err) => {
					log::error!(target: "GraphQL Query Stream", "{err:?}");
					return Poll::Ready(None);
				}
			};

			let (cursor, output) = Query::next(data);
			self.cursor = cursor;

			return Poll::Ready(Some(output));
		}

		if !self.cursor.has_next_page {
			return Poll::Ready(None);
		}

		let variables = Query::update_variables(&self.variables, self.cursor.cursor.clone());
		self.active_query = Some(Query::post(self.client.clone(), variables));

		cx.waker().clone().wake();
		Poll::Pending
	}
}
