use std::str::FromStr;

static SITE_ID: &str = "f48e4964-d583-424b-bace-bd51a12f72a2";
pub use netlify_oauth::*;

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
impl FromStr for OAuthProvider {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"github" => Ok(Self::Github),
			_ => Err(()),
		}
	}
}
