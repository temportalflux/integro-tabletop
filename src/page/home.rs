use yew::prelude::*;

#[function_component]
pub fn Home() -> Html {
	html! {<>
		<crate::components::modal::GeneralPurpose />
		<div>
			{"This is the home page!"}
		</div>
	</>}
}
