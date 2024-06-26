use crate::utility::NotInList;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

static SITE_ID: &str = "f48e4964-d583-424b-bace-bd51a12f72a2";
pub use netlify_oauth::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum OAuthProvider {
	Github,
}

impl OAuthProvider {
	pub fn oauth_id(&self) -> &'static str {
		match self {
			Self::Github => "github",
		}
	}

	pub fn request(&self) -> Request {
		Request {
			site_id: SITE_ID,
			provider_id: self.oauth_id(),
			window_title: "Integro Authorization".into(),
		}
	}
}

impl std::fmt::Display for OAuthProvider {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Github => write!(f, "Github"),
		}
	}
}

impl FromStr for OAuthProvider {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"github" => Ok(Self::Github),
			_ => Err(NotInList(s.to_owned(), vec!["github"])),
		}
	}
}
