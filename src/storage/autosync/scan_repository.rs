use crate::storage::{
	autosync::ModuleFile,
	github::{Error, GetTreeArgs, GithubClient},
};
use std::{collections::VecDeque, path::PathBuf};

pub struct ScanRepository {
	pub status: super::Status,
	pub client: GithubClient,

	pub owner: String,
	pub name: String,
	pub tree_id: Option<String>,
}
impl ScanRepository {
	pub async fn run(self) -> Result<Vec<ModuleFile>, Error> {
		use futures_util::stream::StreamExt;
		let repo_id = format!("{}/{}", self.owner, self.name);
		let mut tree_count = 1;
		self.status.push_stage(format!("Scanning {repo_id}"), Some(tree_count));

		let tree_id = match self.tree_id {
			Some(id) => id,
			None => {
				let mut stream = self.client.search_specific_repos(vec![repo_id.clone()].iter());
				let Some(metadata) = stream.next().await else {
					return Err(Error::InvalidResponse(format!("Missing repository metadata").into()));
				};
				let Some(metadata) = metadata.into_iter().next() else {
					return Err(Error::InvalidResponse(format!("Empty repository metadata").into()));
				};
				metadata.tree_id
			}
		};

		let mut tree_ids = VecDeque::from([(PathBuf::new(), tree_id)]);
		let mut files = Vec::new();
		while let Some((tree_path, tree_id)) = tree_ids.pop_front() {
			let args = GetTreeArgs {
				owner: self.owner.as_str(),
				repo: self.name.as_str(),
				tree_id: tree_id.as_str(),
			};
			self.status.increment_progress();
			for entry in self.client.get_tree(args).await? {
				let full_path = tree_path.join(&entry.path);
				// if the entry is a directory, put it in the queue to be scanned
				if entry.is_tree {
					tree_ids.push_back((full_path, entry.id));
					tree_count += 1;
					self.status.set_progress_max(tree_count);
				} else {
					// only record content files (kdl extension)
					if !entry.path.ends_with(".kdl") {
						continue;
					}
					// extract the system the content is for (which is the top-most parent).
					// if this path has no parent, then it isn't in a system and can be ignored.
					match full_path.parent() {
						None => continue,
						Some(path) if path == std::path::Path::new("") => continue,
						_ => {}
					}
					let system = ModuleFile::get_system_in_file_path(&full_path).unwrap();
					let path_in_repo = full_path.display().to_string().replace("\\", "/");
					files.push(ModuleFile {
						system,
						path_in_repo,
						file_id: entry.id,
					});
				}
			}
		}

		self.status.pop_stage();
		Ok(files)
	}
}
