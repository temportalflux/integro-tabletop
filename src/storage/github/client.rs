use super::{
	queries::{FindOrgs, SearchForRepos, ViewerInfo},
	GraphQLQueryExt, QueryError, QueryStream, RepositoryMetadata,
};
use futures_util::future::LocalBoxFuture;

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
}

pub struct CreateRepo {
	pub org: Option<String>,
	pub name: String,
	pub private: bool,
}
pub struct SetRepoTopics {
	pub owner: String,
	pub repo: String,
	pub topics: Vec<String>,
}
impl GithubClient {
	fn insert_rest_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
		use reqwest::header::*;
		let builder = builder.header(ACCEPT, "application/vnd.github+json");
		let builder = builder.header(AUTHORIZATION, self.1.clone());
		let builder = builder.header("X-Github-Api-Version", "2022-11-28");
		builder
	}

	pub fn create_repo(
		&self,
		request: CreateRepo,
	) -> LocalBoxFuture<'static, anyhow::Result<String>> {
		use serde::Deserialize;
		use serde_json::{Map, Value};
		let builder = self.0.post(match &request.org {
			// create on the authenticated user
			// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
			None => format!("{GITHUB_API}/user/repos"),
			// create on the provided org
			// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-an-organization-repository
			Some(org) => format!("{GITHUB_API}/orgs/{org}/repos"),
		});
		let builder = self.insert_rest_headers(builder);
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
			log::debug!("{json:?}");
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

	pub fn set_repo_topics(
		&self,
		request: SetRepoTopics,
	) -> LocalBoxFuture<'static, anyhow::Result<()>> {
		use serde_json::{Map, Value};
		// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#replace-all-repository-topics
		let builder = self.0.put(format!(
			"{GITHUB_API}/repos/{}/{}/topics",
			request.owner, request.repo
		));
		let builder = self.insert_rest_headers(builder);
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
