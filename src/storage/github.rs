use futures_util::FutureExt;
use graphql_client::GraphQLQuery;
use std::{pin::Pin, task::Poll};

static GITHUB_API_GRAPHQL: &'static str = "https://api.github.com/graphql";
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct GithubClient(reqwest::Client);
impl GithubClient {
	pub fn new(token: &String) -> Result<Self, QueryError> {
		let mut client = reqwest::Client::builder();
		client = client.default_headers({
			let auth_header = format!("Bearer {token}");
			let auth = (
				reqwest::header::AUTHORIZATION,
				reqwest::header::HeaderValue::from_str(&auth_header).unwrap(),
			);
			let agent = (
				reqwest::header::USER_AGENT,
				reqwest::header::HeaderValue::from_str(&APP_USER_AGENT).unwrap(),
			);
			[agent, auth].into_iter().collect()
		});
		let client = client
			.build()
			.map_err(|err| QueryError::ReqwestError(err))?;
		Ok(Self(client))
	}

	pub fn viewer(&self) -> QueryFuture<ViewerInfo> {
		ViewerInfo::post(self.0.clone(), viewer_info::Variables {})
	}

	pub fn find_all_orgs(&self) -> QueryStream<FindOrgs> {
		QueryStream::new(
			self.0.clone(),
			find_orgs::Variables {
				cursor: None,
				amount: 25,
			},
		)
	}

	pub fn search_for_repos(&self, owner: &String) -> QueryStream<SearchForRepos> {
		QueryStream::new(
			self.0.clone(),
			search_for_repos::Variables {
				cursor: None,
				amount: 25,
				query: format!("user:{owner} topic:opensource-tabletop-module"),
			},
		)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
	#[error("request error")]
	ReqwestError(#[from] reqwest::Error),
	#[error("No data in response")]
	NoData,
}
pub type QueryResult<Query> = Result<<Query as GraphQLQuery>::ResponseData, QueryError>;
pub type QueryFuture<Query> = futures_util::future::LocalBoxFuture<'static, QueryResult<Query>>;

trait GraphQLQueryExt<T: GraphQLQuery + 'static> {
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

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/graphql/schema.graphql",
	query_path = "src/storage/github/graphql/viewer.graphql",
	response_derives = "Debug"
)]
pub struct ViewerInfo;

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/graphql/schema.graphql",
	query_path = "src/storage/github/graphql/find_orgs.graphql",
	response_derives = "Debug"
)]
pub struct FindOrgs;

#[derive(Debug)]
pub struct Cursor {
	has_next_page: bool,
	cursor: Option<String>,
}
pub trait StreamableQuery<T: GraphQLQuery> {
	type Item;
	fn update_variables(vars: &T::Variables, cursor: Option<String>) -> T::Variables;
	fn next(data: T::ResponseData) -> (Cursor, Self::Item);
}
impl StreamableQuery<FindOrgs> for FindOrgs {
	type Item = Vec<String>;

	fn update_variables(
		vars: &find_orgs::Variables,
		cursor: Option<String>,
	) -> find_orgs::Variables {
		find_orgs::Variables {
			cursor,
			amount: vars.amount,
		}
	}

	fn next(data: find_orgs::ResponseData) -> (Cursor, Self::Item) {
		let cursor = Cursor {
			has_next_page: data.viewer.organizations.page_info.has_next_page,
			cursor: data.viewer.organizations.page_info.end_cursor,
		};

		let mut output = Vec::new();
		if let Some(org_nodes) = data.viewer.organizations.nodes {
			output.reserve(org_nodes.len());
			for org_node in org_nodes {
				let Some(org) = org_node else { continue; };
				output.push(org.login);
			}
		}

		(cursor, output)
	}
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

	fn poll_next(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Option<Self::Item>> {
		if let Some(mut query) = self.active_query.take() {
			let Poll::Ready(result) = query.poll_unpin(cx) else {
				self.active_query = Some(query);
				return Poll::Pending;
			};
			let data = match result {
				Ok(data) => data,
				Err(err) => {
					log::error!("GraphQL Query failed: {err:?}");
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

type GitObjectID = String;
#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/graphql/schema.graphql",
	query_path = "src/storage/github/graphql/search_for_repos.graphql",
	response_derives = "Debug"
)]
pub struct SearchForRepos;

#[derive(Clone, Debug)]
pub struct RepositoryMetadata {
	pub owner: String,
	pub name: String,
	pub is_private: bool,
	pub version: String,
}

impl StreamableQuery<SearchForRepos> for SearchForRepos {
	type Item = Vec<RepositoryMetadata>;

	fn update_variables(
		vars: &search_for_repos::Variables,
		cursor: Option<String>,
	) -> search_for_repos::Variables {
		search_for_repos::Variables {
			cursor,
			amount: vars.amount,
			query: vars.query.clone(),
		}
	}

	fn next(data: search_for_repos::ResponseData) -> (Cursor, Self::Item) {
		let cursor = Cursor {
			has_next_page: data.search.page_info.has_next_page,
			cursor: data.search.page_info.end_cursor,
		};

		let mut output = Vec::new();
		// rust-analyzer cant read the type data for `nodes` for some reason,
		// use `graphql-client generate --schema-path ./graphql/schema.graphql ./graphql/search_for_repos.graphql`
		// to see the generated structures.
		if let Some(repo_nodes) = data.search.nodes {
			output.reserve(repo_nodes.len());
			for repo_node in repo_nodes {
				let Some(repo) = repo_node else { continue; };
				if let search_for_repos::SearchForReposSearchNodes::Repository(repo) = repo {
					output.push(RepositoryMetadata {
						owner: repo.owner.login,
						name: repo.name,
						is_private: repo.is_private,
						version: repo.default_branch_ref.unwrap().target.oid.to_string(),
					});
				}
			}
		}

		(cursor, output)
	}
}
