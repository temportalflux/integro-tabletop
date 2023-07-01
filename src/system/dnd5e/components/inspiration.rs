use crate::{page::characters::sheet::CharacterHandle, page::characters::sheet::MutatorImpact};
use yew::prelude::*;

#[function_component]
pub fn Inspiration() -> Html {
	let state = use_context::<CharacterHandle>().unwrap();
	let onclick = state.new_dispatch(|_, character| {
		character.inspiration = !character.inspiration;
		MutatorImpact::None
	});
	html! {
		<div class="card m-1 m-xxl-0" style="width: 90px; height: 80px" {onclick}>
			<div class="card-body text-center" style="padding: 5px 5px;">
				<h6 class="card-title" style="font-size: 0.8rem;">{"Inspiration"}</h6>
				<div class="d-flex justify-content-center" style="padding-top: 5px;">
					{state.inspiration().then(|| html! {<div class="icon inspiration" />}).unwrap_or_default()}
				</div>
			</div>
		</div>
	}
}
