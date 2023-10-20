use yew::prelude::*;

pub type SiteId = &'static str;
pub type OAuthId = &'static str;
pub struct Request {
	pub site_id: SiteId,
	pub provider_id: OAuthId,
	pub window_title: String,
}

#[derive(Clone, PartialEq)]
pub struct Auth {
	pub(crate) login: Callback<Request>,
	pub(crate) logout: Callback<()>,
}
impl Auth {
	pub fn login_callback(&self) -> &Callback<Request> {
		&self.login
	}

	pub fn logout_callback(&self) -> &Callback<()> {
		&self.logout
	}

	pub fn sign_in(&self, request: Request) {
		self.login.emit(request);
	}

	pub fn sign_out(&self) {
		self.logout.emit(());
	}
}
