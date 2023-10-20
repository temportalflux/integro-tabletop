use crate::{ChangedFile, ChangedFileStatus, Error, GITHUB_API};
use futures_util::future::LocalBoxFuture;

pub struct Args<'a> {
	pub owner: &'a str,
	pub repo: &'a str,
	// Exclusive start of the range (will not include changes in this commit).
	pub commit_start: &'a str,
	// Inclusive end of the rnage (will include changes in this commit).
	pub commit_end: &'a str,
}

impl crate::GithubClient {
	/// Queries github for the list of files that have changed between two commits.
	/// The list of file path + file ids (sha) within the repository are returned when the request is successful.
	pub fn get_files_changed(&self, request: Args<'_>) -> LocalBoxFuture<'static, Result<Vec<ChangedFile>, Error>> {
		use serde_json::Value;
		use std::str::FromStr;
		// https://docs.github.com/en/rest/commits/commits?apiVersion=2022-11-28#compare-two-commits
		// https://www.git-scm.com/docs/gitrevisions#_dotted_range_notations
		let builder = self.client.get(format!(
			"{GITHUB_API}/repos/{}/{}/compare/{}...{}",
			request.owner, request.repo, request.commit_start, request.commit_end
		));
		let builder = self.insert_rest_headers(builder, None);
		Box::pin(async move {
			let response = builder.send().await?;
			let data = response.json::<serde_json::Value>().await?;
			let Some(Value::Array(entries)) = data.get("files") else {
				return Ok(Vec::new());
			};
			let mut paths_changed = Vec::with_capacity(entries.len());
			for entry in entries {
				let Value::Object(map) = entry else {
					continue;
				};
				let Some(Value::String(file_id)) = map.get("sha") else {
					continue;
				};
				let Some(Value::String(path)) = map.get("filename") else {
					continue;
				};
				let Some(Value::String(status)) = map.get("status") else {
					continue;
				};
				let status = ChangedFileStatus::from_str(status.as_str());
				let status =
					status.map_err(|delta_status| Error::InvalidResponse(format!("{delta_status:?}").into()))?;
				paths_changed.push(ChangedFile {
					path: path.clone(),
					file_id: file_id.clone(),
					status,
				});
			}
			Ok(paths_changed)
		})
	}
}
