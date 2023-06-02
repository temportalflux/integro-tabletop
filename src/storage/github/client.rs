use super::{
	queries::{FindOrgs, SearchForRepos, ViewerInfo},
	GraphQLQueryExt, QueryError, QueryStream, RepositoryMetadata,
};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;

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
	/// The list of file paths within the repository are returned when the request is successful.
	pub fn get_files_changed(
		&self,
		request: FilesChangedArgs<'_>,
	) -> LocalBoxFuture<'static, reqwest::Result<Vec<String>>> {
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
				let Some(Value::String(path)) = map.get("filename") else { continue; };
				paths_changed.push(path.clone());
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
	/// if non-none, then this entry is a directory/tree with some tree_id identifying how to get its contents.
	pub tree_id: Option<String>,
}
impl GithubClient {
	pub fn get_tree(
		&self,
		request: GetTreeArgs<'_>,
	) -> LocalBoxFuture<'static, reqwest::Result<Vec<TreeEntry>>> {
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
				let tree_id = match entry.type_.as_str() {
					"tree" => Some(entry.sha),
					_ => None,
				};
				entries.push(TreeEntry {
					path: entry.path,
					tree_id,
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
