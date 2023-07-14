use crate::{
	components::{Nav, NavDisplay, TabContent},
	page::characters::sheet::ViewProps,
};
use yew::prelude::*;

mod abilities;
pub use abilities::*;
mod class;
pub use class::*;
mod description;
pub use description::*;
mod home;
pub use home::*;
mod origin;
pub use origin::*;

#[function_component]
pub fn Editor(ViewProps { swap_view }: &ViewProps) -> Html {
	let floating_exit_btn = html! {
		<div class="ms-auto">
			<a class="glyph sheet mt-1" onclick={swap_view.reform(|_| ())} />
		</div>
	};
	html! {
		<div class="container overflow-hidden">
			<Nav root_classes={"mt-1"} disp={NavDisplay::Tabs} default_tab_id={"home"} extra={floating_exit_btn}>
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
					<DescriptionTab />
				</TabContent>
			</Nav>
		</div>
	}
}
