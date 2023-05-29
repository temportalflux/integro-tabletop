use crate::storage::github::{Cursor, RepositoryMetadata, StreamableQuery};
use graphql_client::GraphQLQuery;

type GitObjectID = String;

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/queries/graphql/schema.graphql",
	query_path = "src/storage/github/queries/graphql/query_search_for_repos.graphql",
	response_derives = "Debug"
)]
pub struct SearchForRepos;
pub use search_for_repos::Variables;

impl StreamableQuery<SearchForRepos> for SearchForRepos {
	type Item = Vec<RepositoryMetadata>;

	fn update_variables(
		vars: &search_for_repos::Variables,
		cursor: Option<String>,
	) -> search_for_repos::Variables {
		search_for_repos::Variables {
			cursor,
			amount: vars.amount,
			query: vars.query.clone(),
		}
	}

	fn next(data: search_for_repos::ResponseData) -> (Cursor, Self::Item) {
		use search_for_repos::{
			SearchForReposSearchNodes as RepoEnum,
			SearchForReposSearchNodesOnRepositoryObject as Object,
		};
		let cursor = Cursor {
			has_next_page: data.search.page_info.has_next_page,
			cursor: data.search.page_info.end_cursor,
		};

		let mut output = Vec::new();
		// rust-analyzer cant read the type data for `nodes` for some reason,
		// use `graphql-client generate --schema-path ./graphql/schema.graphql ./graphql/search_for_repos.graphql`
		// to see the generated structures.
		if let Some(repo_nodes) = data.search.nodes {
			output.reserve(repo_nodes.len());
			for repo_node in repo_nodes {
				let Some(repo) = repo_node else { continue; };
				let RepoEnum::Repository(repo) = repo else { continue; };
				// All repositories must be initialized (default branch has contents), otherwise they are ignored
				let Some(Object::Tree(default_branch_tree)) = &repo.object else { continue; };
				let Some(root_tree_entries) = &default_branch_tree.entries else { continue; };
				let mut systems = Vec::new();
				for entry in root_tree_entries {
					// if this entry is a directory, then it is likely the root for a system in the module.
					// if its not a tree (directory), then we dont care right now.
					if entry.type_ != "tree" {
						continue;
					}
					systems.push(entry.name.clone());
				}
				output.push(RepositoryMetadata {
					owner: repo.owner.login,
					name: repo.name,
					is_private: repo.is_private,
					version: repo.default_branch_ref.unwrap().target.oid.to_string(),
					systems,
				});
			}
		}

		(cursor, output)
	}
}
