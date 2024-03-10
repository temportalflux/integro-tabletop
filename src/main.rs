use integro_tabletop::*;
use std::sync::Arc;
use yew::prelude::*;

#[cfg(target_family = "wasm")]
fn main() {
	logging::wasm::init(logging::wasm::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}
#[cfg(target_family = "windows")]
fn main() {}

#[function_component]
fn App() -> Html {
	html! {<>
		<ProviderChain>
			<page::App />
		</ProviderChain>
	</>}
}

#[function_component]
fn ProviderChain(props: &html::ChildrenProps) -> Html {
	use crate::components::{mobile, object_browser};
	html! {
		<mobile::Provider threshold={1200}>
			<auth::Provider>
				<task::Provider>
					<system::Provider>
						<DatabaseProvider>
							<storage::autosync::Provider>
								<object_browser::Provider>
									<crate::components::modal::Provider>
										<crate::components::context_menu::Provider>
											{props.children.clone()}
										</crate::components::context_menu::Provider>
									</crate::components::modal::Provider>
								</object_browser::Provider>
							</storage::autosync::Provider>
						</DatabaseProvider>
					</system::Provider>
				</task::Provider>
			</auth::Provider>
		</mobile::Provider>
	}
}

#[function_component]
fn DatabaseProvider(props: &html::ChildrenProps) -> Html {
	use crate::database::Database;
	let database = yew_hooks::use_async(async move {
		match Database::open().await {
			Ok(db) => Ok(db),
			Err(err) => {
				log::error!(target: "tabletop-tools", "Failed to connect to database: {err:?}");
				Err(Arc::new(err))
			}
		}
	});
	// When the app first opens, load the database.
	// Could probably check `use_is_first_mount()`, but checking if there database
	// doesn't exist yet and isn't loading is more clear.
	if database.data.is_none() && !database.loading {
		log::info!(target: "database", "Initializing database");
		database.run();
	}
	// If the database has not yet loaded (or encountered an error),
	// we wont even show the children - mostly to avoid the numerous errors that would occur
	// since children strongly rely on the database existing.
	let Some(ddb) = &database.data else {
		return html!();
	};
	html! {
		<ContextProvider<Database> context={ddb.clone()}>
			{props.children.clone()}
		</ContextProvider<Database>>
	}
}
