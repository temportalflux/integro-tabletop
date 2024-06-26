use database::Record;
use serde::{Deserialize, Serialize};
use std::{str::FromStr};
use crate::data::UserSettings;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Settings {
	pub id: String,
	pub version: String,
	pub remote_version: String,
	pub user_settings: UserSettings,
}

impl Record for Settings {
	fn store_id() -> &'static str {
		"settings"
	}
}
