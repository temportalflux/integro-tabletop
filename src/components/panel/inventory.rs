use crate::{data::ContextMut, system::dnd5e::character::State};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn Inventory() -> Html {
	let state = use_context::<ContextMut<State>>().unwrap();
	let equipped = use_state(|| true);

	// temp demo of mutating another piece of data when a state changes.
	// this will be useful when equipping an item requires rebuilding of
	// features, thereby affecting senses, skills, ability scores, etc.
	use_effect_with_deps(
		move |equipped| {
			let is_equipped = **equipped;
			state.mutate(move |state| {
				if is_equipped {
					state.add_hit_points(1);
				} else {
					state.sub_hit_points(1);
				}
			});
		},
		equipped.clone(),
	);

	html! {<>

		<table class="table table-compact m-0">
			<thead>
				<tr class="text-center" style="font-size: 0.7rem;">
					<th scope="col">{"Equip"}</th>
					<th scope="col">{"Name"}</th>
					<th scope="col">{"Weight"}</th>
					<th scope="col">{"Qty"}</th>
					<th scope="col">{"Notes"}</th>
				</tr>
			</thead>
			<tbody>
				<tr class="align-middle" onclick={Callback::from(|_| log::debug!("TODO: open item interface modal"))}>
					<td class="text-center">
						<input
							class={"form-check-input"} type={"checkbox"}
							checked={*equipped}
							onclick={Callback::from(|evt: web_sys::MouseEvent| evt.stop_propagation())}
							onchange={Callback::from({
								let equipped = equipped.clone();
								move |evt: web_sys::Event| {
									let Some(target) = evt.target() else { return; };
									let Some(input) = target.dyn_ref::<HtmlInputElement>() else { return; };
									log::debug!("equipped state changing");
									equipped.set(input.checked());
								}
							})}
						/>
					</td>
					<td>{"Dagger"}</td>
					<td class="text-center">{1}{" lb."}</td>
					<td class="text-center">{1}</td>
					<td style="max-width: 250px;">{"Simple, Finesse, Light, Thrown, Simple, Finesse, Light, Thrown, Simple, Finesse, Light, Thrown"}</td>
				</tr>
			</tbody>
		</table>

	</>}
}
