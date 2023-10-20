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
	/// The raw content to be uploaded.
	/// Will be base64 encoded before transmitting.
	pub content: &'a str,
	/// If updating a file, this must be some.
	pub file_id: Option<&'a str>,
	/// If omitted, operation will be performed on the default branch.
	pub branch: Option<&'a str>,
}

#[derive(Debug)]
pub struct Response {
	pub file_id: String,
	pub version: String,
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {
	#[error("Requested resource to update was not found.")]
	ResourceNotFound,
	#[error("Requested resource has a merge conflict with the updated content.")]
	FileConflict,
	#[error("Validation failed or operation is being spammed.")]
	ValidationFailed,
	#[error("Unknown response code {0}")]
	Unknown(u16),
}
impl Into<Error> for UpdateError {
	fn into(self) -> Error {
		Error::InvalidResponse(std::sync::Arc::new(format!("{self:?}")))
	}
}

impl crate::GithubClient {
	pub fn create_or_update_file(&self, args: Args<'_>) -> LocalBoxFuture<'static, Result<Response, Error>> {
		use base64ct::{Base64, Encoding};
		use serde_json::{Map, Value};

		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#create-or-update-file-contents
		let Args {
			repo_org,
			repo_name,
			path_in_repo,
			commit_message,
			content,
			file_id,
			branch,
		} = args;

		let path_str = path_in_repo.as_os_str().to_str().unwrap().replace("\\", "/");
		let url = format!("{GITHUB_API}/repos/{repo_org}/{repo_name}/contents/{path_str}");

		// encode using Base64 url-safe compliant with RFC 4648
		// https://stackoverflow.com/a/22962982
		let encoded_content: String = Base64::encode_string(content.as_bytes());

		let builder = self.client.put(url);
		let builder = self.insert_rest_headers(builder, None);
		let body = {
			let mut entries = vec![
				("message".into(), commit_message.to_owned().into()),
				("content".into(), encoded_content.into()),
			];
			if let Some(sha) = file_id {
				entries.push(("sha".into(), sha.to_owned().into()));
			}
			if let Some(branch) = branch {
				entries.push(("branch".into(), branch.to_owned().into()));
			}
			let object = Value::Object(entries.into_iter().collect::<Map<String, Value>>());
			serde_json::to_string(&object).unwrap()
		};
		log::debug!("{body}");
		let builder = builder.body(body);
		Box::pin(async move {
			#[derive(Deserialize)]
			struct Content {
				sha: String,
			}
			#[derive(Deserialize)]
			struct Commit {
				sha: String,
			}
			#[derive(Deserialize)]
			struct ResponseData {
				content: Content,
				commit: Commit,
			}

			let response = builder.send().await?;
			let status = response.status();
			let data = response.json::<Value>().await?;
			match status.as_u16() {
				// 200: file was updated
				// 201: file was created
				200 | 201 => {
					let response = serde_json::from_value::<ResponseData>(data)?;
					let file_id = response.content.sha;
					let version = response.commit.sha;
					Ok(Response { file_id, version })
				}
				404 => {
					log::warn!("{data:?}");
					Err(UpdateError::ResourceNotFound.into())
				}
				409 => {
					log::warn!("{data:?}");
					Err(UpdateError::FileConflict.into())
				}
				422 => {
					log::warn!("{data:?}");
					Err(UpdateError::ValidationFailed.into())
				}
				code => {
					log::warn!("create_or_update_file encountered unknown response code: {code}");
					Err(UpdateError::Unknown(code).into())
				}
			}
		})
	}
}
