use crate::storage::github::RepositoryMetadata;
use graphql_client::GraphQLQuery;

type GitObjectID = String;
#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/queries/graphql/schema.graphql",
	query_path = "src/storage/github/queries/graphql/query_viewer.graphql",
	response_derives = "Debug"
)]
pub struct ViewerInfo;
pub use viewer_info::Variables;

impl ViewerInfo {
	pub fn unpack_repository(
		repo: Option<viewer_info::ViewerInfoViewerRepository>,
	) -> Option<RepositoryMetadata> {
		use viewer_info::ViewerInfoViewerRepositoryDefaultBranchRefTargetRepositoryObject as Object;
		let Some(repo) = repo else { return None; };
		let name = repo.name;
		let owner = repo.owner.login;
		let is_private = repo.is_private;
		// must be initialized with a default branch
		let Some(default_branch_ref) = repo.default_branch_ref else { return None; };
		let version = default_branch_ref.target.oid;

		// HEAD (aka default branch) must have content
		let Some(Object::Tree(head_tree)) = default_branch_ref.target.repository.object else { return None; };
		let tree_id = head_tree.oid;
		let Some(root_tree_entries) = &head_tree.entries else { return None; };
		let mut systems = Vec::new();
		for entry in root_tree_entries {
			// if this entry is a directory, then it is likely the root for a system in the module.
			// if its not a tree (directory), then we dont care right now.
			if entry.type_ != "tree" {
				continue;
			}
			systems.push(entry.name.clone());
		}

		Some(RepositoryMetadata {
			owner,
			name,
			is_private,
			version,
			systems,
			tree_id,
		})
	}
}
