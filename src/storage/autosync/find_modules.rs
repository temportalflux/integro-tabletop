use crate::storage::github::{GithubClient, RepositoryMetadata};

pub struct FindModules {
	pub status: super::Status,
	pub client: GithubClient,
	pub names: Vec<String>,
}
impl FindModules {
	pub async fn run(&mut self) -> Result<Vec<RepositoryMetadata>, crate::storage::github::Error> {
		use futures_util::stream::StreamExt;
		self.status.push_stage("Fetching info on specific modules", None);

		// Regardless of if the homebrew already existed, lets gather ALL of the relevant
		// repositories which are content modules. This will always include the homebrew repo,
		// since it is garunteed to exist due to the above code.
		let mut metadata = Vec::new();
		if !self.names.is_empty() {
			let mut stream = self.client.search_specific_repos(self.names.iter());
			while let Some(repos) = stream.next().await {
				metadata.extend(repos.clone());
			}
		}

		self.status.pop_stage();
		Ok(metadata)
	}
}
