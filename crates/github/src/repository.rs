#[derive(Clone, Debug)]
pub struct RepositoryMetadata {
	pub owner: String,
	pub name: String,
	pub is_private: bool,
	pub version: String,
	pub tree_id: String,
	pub root_trees: Vec<String>,
}
