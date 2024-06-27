use crate::data::UserSettings;
use database::Record;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct UserSettingsRecord {
	pub id: String,
	pub file_id: Option<String>,
	pub version: String,
	pub user_settings: UserSettings,
}

impl Record for UserSettingsRecord {
	fn store_id() -> &'static str {
		"user_settings"
	}
}
