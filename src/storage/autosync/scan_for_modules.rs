use crate::storage::github::{GithubClient, RepositoryMetadata};

pub struct ScanForModules {
	pub status: super::Status,
	pub client: GithubClient,
	pub owners: Vec<String>,
}
impl ScanForModules {
	pub async fn run(self) -> Result<Vec<RepositoryMetadata>, crate::storage::github::Error> {
		use futures_util::stream::StreamExt;
		// Regardless of if the homebrew already existed, lets gather ALL of the relevant
		// repositories which are content modules. This will always include the homebrew repo,
		// since it is garunteed to exist due to the above code.
		let mut metadata = Vec::new();
		let mut stream = self.client.search_for_repos(self.owners.iter());
		while let Some(repos) = stream.next().await {
			metadata.extend(repos.clone());
		}
		Ok(metadata)
	}
}
