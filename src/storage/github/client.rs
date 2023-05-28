use super::{
	queries::{FindOrgs, SearchForRepos, ViewerInfo},
	GraphQLQueryExt, QueryError, QueryFuture, QueryStream,
};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct GithubClient(reqwest::Client);
impl GithubClient {
	pub fn new(token: &String) -> Result<Self, QueryError> {
		let mut client = reqwest::Client::builder();
		client = client.default_headers({
			let auth_header = format!("Bearer {token}");
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
		Ok(Self(client))
	}

	pub fn viewer(&self) -> QueryFuture<ViewerInfo> {
		ViewerInfo::post(self.0.clone(), super::queries::viewer::Variables {})
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
		QueryStream::new(
			self.0.clone(),
			super::queries::search_for_repos::Variables {
				cursor: None,
				amount: 25,
				query: format!("user:{owner} topic:integro-tabletop-module"),
			},
		)
	}
}
