use crate::components::{Nav, NavDisplay, TabContent};
use yew::prelude::*;

mod abilities;
pub use abilities::*;
mod class;
pub use class::*;
mod home;
pub use home::*;
mod origin;
pub use origin::*;

#[derive(Clone, PartialEq, Properties)]
pub struct SheetEditorProps {
	pub open_viewer: Callback<()>,
}

#[function_component]
pub fn SheetEditor(SheetEditorProps { open_viewer }: &SheetEditorProps) -> Html {
	let floating_exit_btn = html! {
		<div class="ms-auto">
			<a class="sheet-icon mt-1" onclick={open_viewer.reform(|_| ())} />
		</div>
	};
	html! {
		<div class="container overflow-hidden">
			<Nav root_classes={""} disp={NavDisplay::Tabs} default_tab_id={"home"} extra={floating_exit_btn}>
				<TabContent id="home" title={html! {{"Home"}}}>
					<HomeTab />
				</TabContent>
				<TabContent id="class" title={html! {{"Class"}}}>
					<ClassTab />
				</TabContent>
				<TabContent id="origin" title={html! {{"Origin"}}}>
					<OriginTab />
				</TabContent>
				<TabContent id="abilities" title={html! {{"Abilities"}}}>
					<AbilitiesTab />
				</TabContent>
				<TabContent id="description" title={html! {{"Description"}}}>
					{"Description"}
				</TabContent>
			</Nav>
		</div>
	}
}
