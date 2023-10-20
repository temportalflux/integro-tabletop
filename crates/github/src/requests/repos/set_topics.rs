use crate::{Error, GITHUB_API};
use futures_util::future::LocalBoxFuture;

pub struct Args<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	pub topics: Vec<String>,
}
impl crate::GithubClient {
	pub fn set_repo_topics(&self, request: Args<'_>) -> LocalBoxFuture<'static, Result<(), Error>> {
		use serde_json::{Map, Value};
		// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#replace-all-repository-topics
		let url = format!("{GITHUB_API}/repos/{}/{}/topics", request.owner, request.repo);
		let builder = self.client.put(url);
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
