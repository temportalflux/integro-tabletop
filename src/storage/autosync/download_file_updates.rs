use crate::{
	storage::autosync::{ModuleFile, ModuleFileUpdate},
	system::{
		self,
		core::{ModuleId, SourceId},
	},
};
use anyhow::Context;
use github::{repos, ChangedFileStatus, Error, GithubClient};
use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

pub struct DownloadFileUpdates {
	pub status: super::Status,
	pub client: GithubClient,
	pub system_depot: system::Depot,

	pub module_id: ModuleId,
	pub version: String,
	pub files: Vec<ModuleFileUpdate>,
}
impl DownloadFileUpdates {
	pub async fn run(mut self) -> Result<(Vec<crate::database::Entry>, HashSet<String>), Error> {
		let ModuleId::Github { user_org, repository } = &self.module_id else {
			// ERROR: Invalid module id
			return Ok((Vec::new(), HashSet::new()));
		};
		self.status
			.push_stage(format!("Downloading {user_org}/{repository}"), Some(self.files.len()));
		let mut entries = Vec::with_capacity(self.files.len());
		let mut removed_file_ids = HashSet::new();
		let files = self.files.drain(..).collect::<Vec<_>>();
		for file_update in files {
			let ModuleFileUpdate {
				file: ModuleFile {
					system,
					path_in_repo,
					file_id,
				},
				status,
			} = file_update;

			self.status.increment_progress();

			let args = repos::contents::get::Args {
				owner: user_org.as_str(),
				repo: repository.as_str(),
				path: Path::new(path_in_repo.as_str()),
				version: self.version.as_str(),
			};
			match status {
				ChangedFileStatus::Added
				| ChangedFileStatus::Modified
				| ChangedFileStatus::Renamed
				| ChangedFileStatus::Copied
				| ChangedFileStatus::Changed => {
					let content = self.client.get_file_content(args).await?;
					let parsed_entries = self.parse_content(system, path_in_repo, file_id, content);
					let parsed_entries =
						parsed_entries.map_err(|err| Error::InvalidResponse(format!("{err:?}").into()))?;
					entries.extend(parsed_entries);
				}
				ChangedFileStatus::Removed => {
					removed_file_ids.insert(file_id);
				}
				ChangedFileStatus::Unchanged => {}
			}
		}
		self.status.pop_stage();
		Ok((entries, removed_file_ids))
	}

	fn parse_content(
		&self,
		system: String,
		file_path: String,
		file_id: String,
		content: String,
	) -> anyhow::Result<Vec<crate::database::Entry>> {
		let Some(system_reg) = self.system_depot.get(&system) else {
			return Ok(Vec::new());
		};

		let document = content
			.parse::<kdl::KdlDocument>()
			.with_context(|| format!("Failed to parse content: {content:?}"))?;
		let path_in_system = match file_path.strip_prefix(&format!("{system}/")) {
			Some(systemless) => PathBuf::from(systemless),
			None => PathBuf::from(&file_path),
		};
		let mut source_id = SourceId {
			module: Some(self.module_id.clone()),
			system: Some(system.clone()),
			path: path_in_system,
			..Default::default()
		};
		let mut entries = Vec::with_capacity(document.nodes().len());
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let category = node.name().value().to_owned();
			let metadata = system_reg.parse_metadata(node, &source_id)?;
			let record = crate::database::Entry {
				id: source_id.to_string(),
				module: self.module_id.to_string(),
				system: system.clone(),
				category: category,
				version: Some(self.version.clone()),
				metadata,
				kdl: node.to_string(),
				file_id: Some(file_id.clone()),
			};
			entries.push(record);
		}
		Ok(entries)
	}
}
