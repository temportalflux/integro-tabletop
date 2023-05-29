use graphql_client::GraphQLQuery;

type GitObjectID = String;
#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/queries/graphql/schema.graphql",
	query_path = "src/storage/github/queries/graphql/query_find_user_repo.graphql",
	response_derives = "Debug"
)]
pub struct FindUserRepo;
pub use find_user_repo::Variables;
