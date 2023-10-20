use crate::{Error, GITHUB_API};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug)]
pub struct Args<'a> {
	pub repo_org: &'a str,
	pub repo_name: &'a str,
	pub path_in_repo: &'a Path,
	pub commit_message: &'a str,
	pub file_id: &'a str,
	/// If omitted, operation will be performed on the default branch.
	pub branch: Option<&'a str>,
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteError {
	#[error("Requested resource to update was not found.")]
	ResourceNotFound,
	#[error("Requested resource has a merge conflict with the updated content.")]
	FileConflict,
	#[error("Validation failed or operation is being spammed.")]
	ValidationFailed,
	#[error("Service unavailable.")]
	ServiceUnavailable,
	#[error("Unknown response code {0}")]
	Unknown(u16),
}
impl Into<Error> for DeleteError {
	fn into(self) -> Error {
		Error::InvalidResponse(std::sync::Arc::new(format!("{self:?}")))
	}
}

impl crate::GithubClient {
	pub fn delete_file(&self, args: Args<'_>) -> LocalBoxFuture<'static, Result<String, Error>> {
		use serde_json::{Map, Value};

		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#delete-a-file
		let Args {
			repo_org,
			repo_name,
			path_in_repo,
			commit_message,
			file_id,
			branch,
		} = args;

		let path_str = path_in_repo.as_os_str().to_str().unwrap().replace("\\", "/");
		let url = format!("{GITHUB_API}/repos/{repo_org}/{repo_name}/contents/{path_str}");

		let builder = self.client.delete(url);
		let builder = self.insert_rest_headers(builder, None);
		let body = {
			let mut entries = vec![
				("message".into(), commit_message.to_owned().into()),
				("sha".into(), file_id.to_owned().into()),
			];
			if let Some(branch) = branch {
				entries.push(("branch".into(), branch.to_owned().into()));
			}
			let object = Value::Object(entries.into_iter().collect::<Map<String, Value>>());
			serde_json::to_string(&object).unwrap()
		};
		let builder = builder.body(body);
		Box::pin(async move {
			#[derive(Deserialize)]
			struct Commit {
				sha: String,
			}
			#[derive(Deserialize)]
			struct Response {
				commit: Commit,
			}

			let response = builder.send().await?;
			let status = response.status();
			let data = response.json::<Value>().await?;
			match status.as_u16() {
				// file was deleted
				200 => {
					let response = serde_json::from_value::<Response>(data)?;
					let version = response.commit.sha;
					Ok(version)
				}
				404 => {
					log::warn!("{data:?}");
					Err(DeleteError::ResourceNotFound.into())
				}
				409 => {
					log::warn!("{data:?}");
					Err(DeleteError::FileConflict.into())
				}
				422 => {
					log::warn!("{data:?}");
					Err(DeleteError::ValidationFailed.into())
				}
				503 => {
					log::warn!("{data:?}");
					Err(DeleteError::ServiceUnavailable.into())
				}
				code => {
					log::warn!("{data:?}");
					Err(DeleteError::Unknown(code).into())
				}
			}
		})
	}
}
