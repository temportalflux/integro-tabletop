use crate::{components::auth, page, storage::autosync, theme};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component]
pub fn App() -> Html {
	let autosync_channel = use_context::<autosync::Channel>().unwrap();
	let autosync_status = use_context::<autosync::Status>().unwrap();
	auth::use_on_auth_success(move |_auth_status| {
		autosync_channel.try_send_req(autosync::Request::FetchLatestVersionAllModules);
	});

	html! {
		<BrowserRouter>
			<Header />
			{match autosync_status.is_active() {
				true => {
					html! {
						<div class="sync-status d-flex justify-content-center align-items-center">
							<div class="d-flex flex-column align-items-center" style="width: 1000px;">
								{autosync_status.stages().iter().enumerate().map(|(idx, stage)| {
									html! {
										<div class="w-100 my-2">
											<div class="d-flex align-items-center">
												{stage.progress.is_none().then(|| {
													html!(<div class="spinner-border me-2" role="status" />)
												})}
												<div class={format!("h{}", idx+1)}>{&stage.title}</div>
											</div>
											{stage.progress.as_ref().map(|status| {
												let progress = (status.progress as f64 / status.max as f64) * 100f64;
												html! {
													<div class="progress" role="progressbar">
														<div class="progress-bar bg-success" style={format!("width: {progress}%")}>
															{status.progress} {"/"} {status.max}
														</div>
													</div>
												}
											})}
										</div>
									}
								}).collect::<Vec<_>>()}
							</div>
						</div>
					}
				}
				false => html!(<Switch<Route> render={Route::switch} />),
			}}

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
	let autosync_status = use_context::<autosync::Status>().unwrap();
	let cls_disabled = autosync_status.is_active().then_some("disabled");
	let auth_content = html!(<auth::LoginButton />);
	html! {
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<Link<Route> classes={classes!("navbar-brand", cls_disabled)} to={Route::Home}>{"Integro Tabletop"}</Link<Route>>
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
								<Link<Route>
									classes={classes!("nav-link", cls_disabled)}
									to={Route::Characters}
								>{"My Characters"}</Link<Route>>
							</li>
							<li class="nav-item">
								<Link<Route>
									classes={classes!("nav-link", cls_disabled)}
									to={Route::Modules}
								>{"Modules"}</Link<Route>>
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
