use super::{
	queries::{FindOrgs, SearchForRepos, ViewerInfo},
	GraphQLQueryExt, QueryError, QueryStream, RepositoryMetadata,
};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;
use std::path::Path;

static GITHUB_API: &'static str = "https://api.github.com";
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct GithubClient(reqwest::Client, String);
impl GithubClient {
	pub fn new(token: &String) -> Result<Self, QueryError> {
		let mut client = reqwest::Client::builder();
		let auth_header = format!("Bearer {token}");
		client = client.default_headers({
			let auth = (
				reqwest::header::AUTHORIZATION,
				reqwest::header::HeaderValue::from_str(&auth_header).unwrap(),
			);
			let agent = (
				reqwest::header::USER_AGENT,
				reqwest::header::HeaderValue::from_str(&APP_USER_AGENT).unwrap(),
			);
			[agent, auth].into_iter().collect()
		});
		let client = client
			.build()
			.map_err(|err| QueryError::ReqwestError(err))?;
		Ok(Self(client, auth_header))
	}

	pub fn viewer(
		&self,
	) -> LocalBoxFuture<'static, anyhow::Result<(String, Option<RepositoryMetadata>)>> {
		use crate::storage::USER_HOMEBREW_REPO_NAME;
		let client = self.0.clone();
		Box::pin(async move {
			let response = ViewerInfo::post(
				client,
				super::queries::viewer::Variables {
					repo_name: USER_HOMEBREW_REPO_NAME.to_owned(),
				},
			)
			.await?;
			let repository = ViewerInfo::unpack_repository(response.viewer.repository);
			Ok((response.viewer.login, repository))
		})
	}

	pub fn find_all_orgs(&self) -> QueryStream<FindOrgs> {
		QueryStream::new(
			self.0.clone(),
			super::queries::find_orgs::Variables {
				cursor: None,
				amount: 25,
			},
		)
	}

	pub fn search_for_repos(&self, owner: &String) -> QueryStream<SearchForRepos> {
		use super::MODULE_TOPIC;
		QueryStream::new(
			self.0.clone(),
			super::queries::search_for_repos::Variables {
				cursor: None,
				amount: 25,
				query: format!("user:{owner} topic:{MODULE_TOPIC}"),
			},
		)
	}

	fn insert_rest_headers(
		&self,
		builder: reqwest::RequestBuilder,
		media_type: Option<&'static str>,
	) -> reqwest::RequestBuilder {
		use reqwest::header::*;
		let accept = match media_type {
			None => format!("application/vnd.github+json"),
			Some(media) => format!("application/vnd.github.{media}+json"),
		};
		let builder = builder.header(ACCEPT, accept);
		let builder = builder.header(AUTHORIZATION, self.1.clone());
		let builder = builder.header("X-Github-Api-Version", "2022-11-28");
		builder
	}
}

pub struct CreateRepoArgs<'a> {
	pub org: Option<&'a str>,
	pub name: &'a str,
	pub private: bool,
}
impl GithubClient {
	pub fn create_repo(
		&self,
		request: CreateRepoArgs<'_>,
	) -> LocalBoxFuture<'static, anyhow::Result<String>> {
		use serde_json::{Map, Value};
		let builder = self.0.post(match request.org {
			// create on the authenticated user
			// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
			None => format!("{GITHUB_API}/user/repos"),
			// create on the provided org
			// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-an-organization-repository
			Some(org) => format!("{GITHUB_API}/orgs/{org}/repos"),
		});
		let builder = self.insert_rest_headers(builder, None);
		let builder = builder.body(
			serde_json::to_string(&Value::Object(
				[
					("name".into(), request.name.clone().into()),
					("private".into(), request.private.into()),
					("auto_init".into(), true.into()),
				]
				.into_iter()
				.collect::<Map<String, Value>>(),
			))
			.unwrap(),
		);
		Box::pin(async move {
			let response = builder.send().await?;
			let json = response.json::<Value>().await?;
			#[derive(Deserialize)]
			struct Data {
				owner: Owner,
			}
			#[derive(Deserialize)]
			struct Owner {
				login: String,
			}
			let data = serde_json::from_value::<Data>(json)?;
			Ok(data.owner.login)
		})
	}
}

pub struct SetRepoTopicsArgs<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	pub topics: Vec<String>,
}
impl GithubClient {
	pub fn set_repo_topics(
		&self,
		request: SetRepoTopicsArgs<'_>,
	) -> LocalBoxFuture<'static, anyhow::Result<()>> {
		use serde_json::{Map, Value};
		// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#replace-all-repository-topics
		let builder = self.0.put(format!(
			"{GITHUB_API}/repos/{}/{}/topics",
			request.owner, request.repo
		));
		let builder = self.insert_rest_headers(builder, None);
		let builder = builder.body(
			serde_json::to_string(&Value::Object(
				[("names".into(), request.topics.into())]
					.into_iter()
					.collect::<Map<String, Value>>(),
			))
			.unwrap(),
		);
		Box::pin(async move {
			let response = builder.send().await?;
			let _data = response.json::<Value>().await?;
			//log::debug!("{data:?}");
			Ok(())
		})
	}
}

pub struct FilesChangedArgs<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	// Exclusive start of the range (will not include changes in this commit).
	pub commit_start: &'a str,
	// Inclusive end of the rnage (will include changes in this commit).
	pub commit_end: &'a str,
}
impl GithubClient {
	/// Queries github for the list of files that have changed between two commits.
	/// The list of file path + file ids (sha) within the repository are returned when the request is successful.
	pub fn get_files_changed(
		&self,
		request: FilesChangedArgs<'_>,
	) -> LocalBoxFuture<'static, reqwest::Result<Vec<(String, String)>>> {
		use serde_json::Value;
		// https://docs.github.com/en/rest/commits/commits?apiVersion=2022-11-28#compare-two-commits
		// https://www.git-scm.com/docs/gitrevisions#_dotted_range_notations
		let builder = self.0.get(format!(
			"{GITHUB_API}/repos/{}/{}/compare/{}...{}",
			request.owner, request.repo, request.commit_start, request.commit_end
		));
		let builder = self.insert_rest_headers(builder, None);
		Box::pin(async move {
			let response = builder.send().await?;
			let data = response.json::<serde_json::Value>().await?;
			let Some(Value::Array(entries)) = data.get("files") else { return Ok(Vec::new()); };
			let mut paths_changed = Vec::with_capacity(entries.len());
			for entry in entries {
				let Value::Object(map) = entry else { continue; };
				let Some(Value::String(file_id)) = map.get("sha") else { continue; };
				let Some(Value::String(path)) = map.get("filename") else { continue; };
				paths_changed.push((path.clone(), file_id.clone()));
			}
			Ok(paths_changed)
		})
	}
}

pub struct GetTreeArgs<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	pub tree_id: &'a str,
}
#[derive(Debug)]
pub struct TreeEntry {
	pub path: String,
	/// The sha of the entry.
	/// If this is a tree/dir, this is used to get the contents of the dir.
	/// If this is a file, this is the file id / sha used to update contents.
	pub id: String,
	pub is_tree: bool,
}
impl GithubClient {
	pub fn get_tree(
		&self,
		request: GetTreeArgs<'_>,
	) -> LocalBoxFuture<'static, reqwest::Result<Vec<TreeEntry>>> {
		// https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28#get-a-tree
		let builder = self.0.get(format!(
			"{GITHUB_API}/repos/{}/{}/git/trees/{}",
			request.owner, request.repo, request.tree_id
		));
		let builder = self.insert_rest_headers(builder, None);
		Box::pin(async move {
			#[derive(Deserialize)]
			struct Tree {
				tree: Vec<Entry>,
			}
			#[derive(Deserialize)]
			struct Entry {
				path: String,
				sha: String,
				#[serde(rename = "type")]
				type_: String,
			}
			let response = builder.send().await?;
			let data = response.json::<Tree>().await?;
			let mut entries = Vec::with_capacity(data.tree.len());
			for entry in data.tree {
				entries.push(TreeEntry {
					path: entry.path,
					is_tree: entry.type_.as_str() == "tree",
					id: entry.sha,
				});
			}
			Ok(entries)
		})
	}
}

pub struct FileContentArgs<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	/// The path to the file in the repository.
	pub path: &'a str,
	pub version: &'a str,
}
impl GithubClient {
	/// Fetches the raw content of a file in a repository.
	pub fn get_file_content(
		&self,
		request: FileContentArgs<'_>,
	) -> LocalBoxFuture<'static, reqwest::Result<String>> {
		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#get-repository-content
		// https://docs.github.com/en/rest/overview/media-types?apiVersion=2022-11-28
		let builder = self.0.get(format!(
			"{GITHUB_API}/repos/{}/{}/contents/{}?ref={}",
			request.owner, request.repo, request.path, request.version,
		));
		let builder = self.insert_rest_headers(builder, Some("raw"));
		Box::pin(async move {
			let response = builder.send().await?;
			Ok(response.text().await?)
		})
	}
}

#[derive(Debug)]
pub struct CreateOrUpdateFileArgs<'a> {
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
pub struct CreateOrUpdateFileResponse {
	pub file_id: String,
	pub version: String,
}
#[derive(thiserror::Error, Debug)]
pub enum CreateOrUpdateFileError {
	#[error("Requested resource to update was not found.")]
	ResourceNotFound,
	#[error("Requested resource has a merge conflict with the updated content.")]
	FileConflict,
	#[error("Validation failed or operation is being spammed.")]
	ValidationFailed,
	#[error("Unknown response code {0}")]
	Unknown(u16),
}
impl GithubClient {
	pub fn create_or_update_file(
		&self,
		args: CreateOrUpdateFileArgs<'_>,
	) -> LocalBoxFuture<'static, anyhow::Result<CreateOrUpdateFileResponse>> {
		use base64ct::{Base64, Encoding};
		use serde_json::{Map, Value};

		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#create-or-update-file-contents
		let CreateOrUpdateFileArgs {
			repo_org,
			repo_name,
			path_in_repo,
			commit_message,
			content,
			file_id,
			branch,
		} = args;

		let path_str = path_in_repo
			.as_os_str()
			.to_str()
			.unwrap()
			.replace("\\", "/");
		let url = format!("{GITHUB_API}/repos/{repo_org}/{repo_name}/contents/{path_str}");

		// encode using Base64 url-safe compliant with RFC 4648
		// https://stackoverflow.com/a/22962982
		let encoded_content: String = Base64::encode_string(content.as_bytes());

		let builder = self.0.put(url);
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
			struct Response {
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
					let response = serde_json::from_value::<Response>(data)?;
					let file_id = response.content.sha;
					let version = response.commit.sha;
					Ok(CreateOrUpdateFileResponse { file_id, version })
				}
				404 => {
					log::warn!("{data:?}");
					Err(CreateOrUpdateFileError::ResourceNotFound.into())
				}
				409 => {
					log::warn!("{data:?}");
					Err(CreateOrUpdateFileError::FileConflict.into())
				}
				422 => {
					log::warn!("{data:?}");
					Err(CreateOrUpdateFileError::ValidationFailed.into())
				}
				code => {
					log::warn!("create_or_update_file encountered unknown response code: {code}");
					Err(CreateOrUpdateFileError::Unknown(code).into())
				}
			}
		})
	}
}

#[derive(Debug)]
pub struct DeleteFileArgs<'a> {
	pub repo_org: &'a str,
	pub repo_name: &'a str,
	pub path_in_repo: &'a Path,
	pub commit_message: &'a str,
	pub file_id: &'a str,
	/// If omitted, operation will be performed on the default branch.
	pub branch: Option<&'a str>,
}
#[derive(thiserror::Error, Debug)]
pub enum DeleteFileError {
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
impl GithubClient {
	pub fn delete_file(
		&self,
		args: DeleteFileArgs<'_>,
	) -> LocalBoxFuture<'static, anyhow::Result<String>> {
		use serde_json::{Map, Value};

		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#delete-a-file
		let DeleteFileArgs {
			repo_org,
			repo_name,
			path_in_repo,
			commit_message,
			file_id,
			branch,
		} = args;

		let path_str = path_in_repo
			.as_os_str()
			.to_str()
			.unwrap()
			.replace("\\", "/");
		let url = format!("{GITHUB_API}/repos/{repo_org}/{repo_name}/contents/{path_str}");

		let builder = self.0.delete(url);
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
					Err(DeleteFileError::ResourceNotFound.into())
				}
				409 => {
					log::warn!("{data:?}");
					Err(DeleteFileError::FileConflict.into())
				}
				422 => {
					log::warn!("{data:?}");
					Err(DeleteFileError::ValidationFailed.into())
				}
				503 => {
					log::warn!("{data:?}");
					Err(DeleteFileError::ServiceUnavailable.into())
				}
				code => {
					log::warn!("{data:?}");
					Err(DeleteFileError::Unknown(code).into())
				}
			}
		})
	}
}
