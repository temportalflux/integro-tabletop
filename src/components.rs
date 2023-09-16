mod annotated_number;
pub use annotated_number::*;

pub mod auth;
pub mod context_menu;
pub mod database;
mod media_query;
pub use media_query::*;
pub mod mobile;
pub mod modal;
mod nav;
pub use nav::*;
pub mod object_browser;
pub mod progress_bar;
mod spinner;
pub use spinner::*;
mod style;
pub use style::*;
mod tag;
pub use tag::*;
mod view_scaler;
pub use view_scaler::*;

pub fn stop_propagation() -> yew::prelude::Callback<web_sys::MouseEvent> {
	yew::prelude::Callback::from(|evt: web_sys::MouseEvent| evt.stop_propagation())
}
