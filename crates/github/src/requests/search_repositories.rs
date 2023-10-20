use crate::{queries, GithubClient, QueryStream, RepositoryMetadata};
use futures_util::future::LocalBoxFuture;
use itertools::Itertools;

#[derive(Default)]
pub struct Query(Vec<(Option<&'static str>, String)>);
impl Query {
	pub fn value(mut self, value: impl Into<String>) -> Self {
		self.0.push((None, value.into()));
		self
	}

	pub fn keyed(mut self, key: &'static str, value: impl Into<String>) -> Self {
		self.0.push((Some(key), value.into()));
		self
	}
}
impl ToString for Query {
	fn to_string(&self) -> String {
		self.0
			.iter()
			.map(|(key, val)| match key {
				Some(key) => format!("{key}:{val}"),
				None => val.clone(),
			})
			.join(" ")
	}
}

pub struct SearchRepositoriesParams {
	pub query: Query,
	pub page_size: usize,
}

impl GithubClient {
	pub fn search_repositories(
		&self,
		params: SearchRepositoriesParams,
	) -> LocalBoxFuture<'static, (String, Vec<RepositoryMetadata>)> {
		let query = params.query.to_string();
		log::debug!(target: "github", "search query {query:?}");
		let mut stream = QueryStream::<queries::SearchForRepos>::new(
			self.client.clone(),
			queries::search_for_repos::Variables {
				cursor: None,
				amount: params.page_size as i64,
				query,
			},
		);

		Box::pin(async move {
			use futures_util::StreamExt;
			let mut viewer = String::default();
			let mut repositories = Vec::new();
			while let Some(mut page) = stream.next().await {
				repositories.append(&mut page.repositories);
				viewer = page.viewer;
			}
			log::debug!(target: "github", "search result {viewer:?} {repositories:?}");
			(viewer, repositories)
		})
	}
}
