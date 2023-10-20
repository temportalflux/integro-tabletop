use github::Query;

use crate::storage::{
	github::{GithubClient, RepositoryMetadata, SearchRepositoriesParams},
	MODULE_TOPIC,
};

pub struct FindModules {
	pub status: super::Status,
	pub client: GithubClient,
	pub names: Vec<String>,
}
impl FindModules {
	pub async fn run(&mut self) -> Result<Vec<RepositoryMetadata>, github::Error> {
		self.status.push_stage("Fetching info on specific modules", None);

		let iter_reponames = self.names.iter().cloned();
		let query = iter_reponames.fold(Query::default(), |query, name| query.keyed("repo", name));
		let query = query.keyed("topic", MODULE_TOPIC);

		let search_params = SearchRepositoriesParams { query, page_size: 25 };
		let (_, repositories) = self.client.search_repositories(search_params).await;

		self.status.pop_stage();

		Ok(repositories)
	}
}
