use yew::prelude::*;

#[function_component]
pub fn NotFound() -> Html {
	html! {<>
		<crate::components::modal::GeneralPurpose />
		{"The page you are looking for does not exist."}
	</>}
}
