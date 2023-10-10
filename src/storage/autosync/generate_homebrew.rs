use crate::storage::{
	github::{CreateRepoArgs, GithubClient, SetRepoTopicsArgs},
	USER_HOMEBREW_REPO_NAME,
};

// Create the homebrew repo on the github client viewer (the user that is logged in).
// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#create-or-update-file-contents
pub struct GenerateHomebrew {
	pub status: super::Status,
	pub client: GithubClient,
}
impl GenerateHomebrew {
	pub async fn run(self) -> Result<(), crate::storage::github::Error> {
		self.status.activate_with_title("Initializing homebrew module", None);

		use crate::storage::github::MODULE_TOPIC;
		let create_repo = CreateRepoArgs {
			org: None,
			name: USER_HOMEBREW_REPO_NAME,
			private: true,
		};
		let owner = self.client.create_repo(create_repo).await?;

		let set_topics = SetRepoTopicsArgs {
			owner: owner.as_str(),
			repo: USER_HOMEBREW_REPO_NAME,
			topics: vec![MODULE_TOPIC.to_owned()],
		};
		self.client.set_repo_topics(set_topics).await?;

		self.status.deactivate();
		Ok(())
	}
}
