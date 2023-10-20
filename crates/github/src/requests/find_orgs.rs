use crate::{queries, GithubClient, QueryStream};
use futures_util::future::LocalBoxFuture;

impl GithubClient {
	pub fn find_orgs(&self) -> LocalBoxFuture<'static, Vec<String>> {
		let mut stream = QueryStream::<queries::FindOrgs>::new(
			self.client.clone(),
			queries::find_orgs::Variables {
				cursor: None,
				amount: 25,
			},
		);
		Box::pin(async move {
			use futures_util::StreamExt;
			let mut owners = Vec::new();
			while let Some(org_list) = stream.next().await {
				owners.extend(org_list);
			}
			owners
		})
	}
}
