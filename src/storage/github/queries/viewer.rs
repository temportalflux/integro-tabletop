use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
	schema_path = "src/storage/github/queries/graphql/schema.graphql",
	query_path = "src/storage/github/queries/graphql/viewer.graphql",
	response_derives = "Debug"
)]
pub struct ViewerInfo;
pub use viewer_info::Variables;
