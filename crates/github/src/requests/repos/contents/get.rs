use crate::{Error, GITHUB_API};
use futures_util::future::LocalBoxFuture;
use std::path::Path;

pub struct Args<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	/// The path to the file in the repository.
	pub path: &'a Path,
	pub version: &'a str,
}

impl crate::GithubClient {
	/// Fetches the raw content of a file in a repository.
	pub fn get_file_content(&self, request: Args<'_>) -> LocalBoxFuture<'static, Result<String, Error>> {
		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#get-repository-content
		// https://docs.github.com/en/rest/overview/media-types?apiVersion=2022-11-28
		let path_str = request.path.as_os_str().to_str().unwrap().replace("\\", "/");
		let builder = self.client.get(format!(
			"{GITHUB_API}/repos/{}/{}/contents/{}?ref={}",
			request.owner, request.repo, path_str, request.version,
		));
		let builder = self.insert_rest_headers(builder, Some("raw"));
		Box::pin(async move {
			let response = builder.send().await?;
			Ok(response.text().await?)
		})
	}
}
