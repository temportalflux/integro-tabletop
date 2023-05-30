use crate::{components::auth, page, theme};
use yew::prelude::*;
use yew_router::{prelude::Redirect, BrowserRouter, Routable, Switch};

#[function_component]
pub fn App() -> Html {
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
	CharactersRoot,
	#[at("/characters/*")]
	Characters,
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
			Self::Modules => html!(<page::OwnedModules />),
			Self::CharactersRoot | Self::Characters => html!(<page::characters::Switch />),
			Self::NotFound => html!(<page::NotFound />),
		}
	}
}

#[function_component]
fn Header() -> Html {
	//let auth_content = html!();
	let auth_content = html!(<auth::LoginButton />);
	html! {
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<a class="navbar-brand" href="/">{"Tabletop Tools"}</a>
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
								<a class="nav-link">{"My Characters"}</a>
							</li>
							<li class="nav-item">
								<a class="nav-link">{"Content Browser"}</a>
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
