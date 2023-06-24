mod annotated_number;
pub use annotated_number::*;

pub mod auth;
pub mod database;
pub mod modal;
mod nav;
pub use nav::*;
pub mod object_browser;
mod spinner;
pub use spinner::*;
mod tag;
pub use tag::*;

pub fn stop_propagation() -> yew::prelude::Callback<web_sys::MouseEvent> {
	yew::prelude::Callback::from(|evt: web_sys::MouseEvent| evt.stop_propagation())
}
