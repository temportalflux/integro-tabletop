use crate::{components::auth, page, storage::autosync, theme};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component]
pub fn App() -> Html {
	let autosync_channel = use_context::<autosync::Channel>().unwrap();
	auth::use_on_auth_success(move |_auth_status| {
		log::debug!(target: "autosync", "Successful auth, poke storage for latest versions of all installed modules");
		autosync_channel.try_send_req(autosync::Request::FetchLatestVersionAllModules);
	});

	html! {
		<BrowserRouter>
			<Header />
			<Switch<Route> render={Route::switch} />
		</BrowserRouter>
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
	#[at("/")]
	Home,
	#[at("/modules")]
	Modules,
	#[at("/characters")]
	Characters,
	#[at("/characters/*")]
	CharacterSheets,
	#[not_found]
	#[at("/404")]
	NotFound,
}

impl Route {
	pub fn not_found() -> Html {
		html!(<Redirect<Self> to={Self::NotFound} />)
	}

	fn switch(self) -> Html {
		match self {
			Self::Home => html!(<page::Home />),
			Self::Modules => html!(<page::ModulesLanding />),
			Self::Characters | Self::CharacterSheets => html!(<page::characters::Switch />),
			Self::NotFound => html!(<page::NotFound />),
		}
	}
}

#[function_component]
fn Header() -> Html {
	let auth_content = html!(<auth::LoginButton />);
	html! {
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<Link<Route> classes="navbar-brand" to={Route::Home}>{"Integro Tabletop"}</Link<Route>>
					<button
						class="navbar-toggler" type="button"
						data-bs-toggle="collapse" data-bs-target="#navContent"
						aria-controls="navContent" aria-expanded="false" aria-label="Toggle navigation"
					>
						<span class="navbar-toggler-icon"></span>
					</button>
					<div class="collapse navbar-collapse" id="navContent">
						<ul class="navbar-nav">
							<li class="nav-item">
								<Link<Route> classes="nav-link" to={Route::Characters}>{"My Characters"}</Link<Route>>
							</li>
							<li class="nav-item">
								<Link<Route> classes="nav-link" to={Route::Modules}>{"Modules"}</Link<Route>>
							</li>
						</ul>
						<ul class="navbar-nav flex-row flex-wrap ms-md-auto">
							<theme::Dropdown />
							{auth_content}
						</ul>
					</div>
				</div>
			</nav>
		</header>
	}
}
