//! Also inspired by https://github.com/XAMPPRocky/octocrab, which does not support WASM.

mod error;
pub use error::*;
pub mod queries;
mod query;
pub use query::*;
mod requests;
pub use requests::*;
mod repository;
pub use repository::*;
mod changed_file;
pub use changed_file::*;

pub(crate) static GITHUB_API: &'static str = "https://api.github.com";

#[derive(Clone)]
pub struct GithubClient {
	pub(crate) client: reqwest::Client,
	pub(crate) auth_header: String,
}

impl GithubClient {
	pub fn new(token: &String, user_agent: &'static str) -> Result<Self, Error> {
		let mut client = reqwest::Client::builder();
		let auth_header = format!("Bearer {token}");
		client = client.default_headers({
			let auth = (
				reqwest::header::AUTHORIZATION,
				reqwest::header::HeaderValue::from_str(&auth_header).unwrap(),
			);
			let agent = (
				reqwest::header::USER_AGENT,
				reqwest::header::HeaderValue::from_str(user_agent).unwrap(),
			);
			[agent, auth].into_iter().collect()
		});
		let client = client.build()?;
		Ok(Self { client, auth_header })
	}

	pub(crate) fn insert_rest_headers(
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
		let builder = builder.header(AUTHORIZATION, self.auth_header.clone());
		let builder = builder.header("X-Github-Api-Version", "2022-11-28");
		builder
	}
}
