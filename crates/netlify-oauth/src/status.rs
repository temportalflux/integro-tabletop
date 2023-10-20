use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
pub enum Status {
	#[default]
	None,
	Authorizing,
	Successful {
		oauth_id: String,
		token: String,
	},
	Failed {
		error: String,
	},
}
