use crate::{Error, GITHUB_API};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;

pub struct Args<'a> {
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

impl crate::GithubClient {
	pub fn get_tree(&self, request: Args<'_>) -> LocalBoxFuture<'static, Result<Vec<TreeEntry>, Error>> {
		// https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28#get-a-tree
		let builder = self.client.get(format!(
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
