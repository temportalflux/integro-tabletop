use crate::{
	storage::autosync::{ModuleFile, ModuleFileUpdate},
	system::{self, ModuleId, SourceId},
};
use anyhow::Context;
use github::{repos, ChangedFileStatus, Error, GithubClient};
use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

// One node per document is required for 2 reasons:
// 1. Updating the content of a file is much more complicated if there are more than 1 entry per document,
//    because other content has to either be reparsed or fetched from database for any one entry to be written.
// 2. Variants need a way to document that they are not the original object they are based on,
//    without creating a whole new id. The marker for this replaced the old `node_idx` field in the source id.
#[derive(thiserror::Error, Debug)]
#[error("Too many entries in document {0}, only one node per document is permitted.")]
pub struct TooManyEntries(pub String);

pub struct DownloadFileUpdates {
	pub status: super::Status,
	pub client: GithubClient,
	pub system_depot: system::Registry,

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

			if !path_in_repo.ends_with(".kdl") {
				continue;
			}

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
					let parsed_record = self.parse_content(system, path_in_repo, file_id, content);
					let parsed_record =
						parsed_record.map_err(|err| Error::InvalidResponse(format!("{err:?}").into()))?;
					if let Some(record) = parsed_record {
						entries.push(record);
					}
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
		&self, system: String, file_path: String, file_id: String, content: String,
	) -> anyhow::Result<Option<crate::database::Entry>> {
		let Some(system_reg) = self.system_depot.get(&system) else {
			return Ok(None);
		};

		let document = content
			.parse::<kdl::KdlDocument>()
			.with_context(|| format!("Failed to parse content: {content:?}"))?;
		let path_in_system = match file_path.strip_prefix(&format!("{system}/")) {
			Some(systemless) => PathBuf::from(systemless),
			None => PathBuf::from(&file_path),
		};
		let source_id = SourceId {
			module: Some(self.module_id.clone()),
			system: Some(system.clone()),
			path: path_in_system,
			..Default::default()
		};
		if document.nodes().len() > 1 {
			return Err(TooManyEntries(source_id.to_string()).into());
		}
		let Some(node) = document.nodes().first() else {
			return Ok(None);
		};

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
			generator_id: None,
			generated: 0,
		};

		Ok(Some(record))
	}
}
