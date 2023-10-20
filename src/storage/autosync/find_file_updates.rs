use crate::storage::autosync::{ModuleFile, ModuleFileUpdate};
use github::{repos, GithubClient};

pub struct FindFileUpdates {
	pub status: super::Status,
	pub client: GithubClient,

	pub owner: String,
	pub name: String,
	pub old_version: String,
	pub new_version: String,
}
impl FindFileUpdates {
	pub async fn run(self) -> Result<Vec<ModuleFileUpdate>, github::Error> {
		// Getting the files changed for this upgrade
		let args = repos::compare::Args {
			owner: self.owner.as_str(),
			repo: self.name.as_str(),
			commit_start: self.old_version.as_str(),
			commit_end: self.new_version.as_str(),
		};

		let changed_file_paths = self.client.get_files_changed(args).await?;
		let mut files = Vec::with_capacity(changed_file_paths.len());
		for changed_file in changed_file_paths {
			let path_in_repo = std::path::Path::new(&changed_file.path);
			let system = ModuleFile::get_system_in_file_path(path_in_repo).unwrap();
			files.push(ModuleFileUpdate {
				file: ModuleFile {
					system,
					path_in_repo: changed_file.path,
					file_id: changed_file.file_id,
				},
				status: changed_file.status,
			});
		}

		Ok(files)
	}
}
