use crate::storage::github::{GithubClient, RepositoryMetadata};
use std::collections::BTreeMap;

pub struct FindModules {
	pub status: super::Status,
	pub client: GithubClient,
	pub names: Vec<String>,
}
impl FindModules {
	pub async fn run(self) -> Result<Vec<RepositoryMetadata>, crate::storage::github::Error> {
		use futures_util::stream::StreamExt;
		self.status
			.activate_with_title("Fetching info on specific modules", None);

		// Regardless of if the homebrew already existed, lets gather ALL of the relevant
		// repositories which are content modules. This will always include the homebrew repo,
		// since it is garunteed to exist due to the above code.
		let mut relevant_list = BTreeMap::new();
		let mut metadata = Vec::new();
		let mut stream = self.client.search_specific_repos(self.names.iter());
		while let Some(repos) = stream.next().await {
			metadata.extend(repos.clone());
			for repo in repos {
				relevant_list.insert((repo.owner, repo.name), (repo.is_private, repo.version));
			}
		}

		self.status.deactivate();
		Ok(metadata)
	}
}
