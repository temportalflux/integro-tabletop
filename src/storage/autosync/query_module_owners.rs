use crate::storage::github::GithubClient;

// Query github for the logged in user and all organizations they have access to.
pub struct QueryModuleOwners {
	pub status: super::Status,
	pub client: GithubClient,
	pub found_homebrew: bool,
}
impl QueryModuleOwners {
	pub async fn run(&mut self) -> Result<Vec<String>, crate::storage::github::Error> {
		use futures_util::stream::StreamExt;
		self.status
			.activate_with_title("Searching storage for module owners", None);
		let (user, homebrew_repo) = self.client.viewer().await?;

		let mut owners = vec![user.clone()];
		let mut find_all_orgs = self.client.find_all_orgs();
		while let Some(org_list) = find_all_orgs.next().await {
			owners.extend(org_list);
		}

		self.found_homebrew = homebrew_repo.is_some();

		self.status.deactivate();
		Ok(owners)
	}
}
