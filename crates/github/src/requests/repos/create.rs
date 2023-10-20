use crate::{Error, GITHUB_API};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;

pub struct Args<'a> {
	pub org: Option<&'a str>,
	pub name: &'a str,
	pub private: bool,
}

impl crate::GithubClient {
	pub fn create_repo(&self, request: Args<'_>) -> LocalBoxFuture<'static, Result<String, Error>> {
		use serde_json::{Map, Value};
		let builder = self.client.post(match request.org {
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
