use std::{rc::Rc, str::FromStr};

use crate::{
	components::{
		database::{use_query_all_typed, use_typed_fetch_callback, QueryAllArgs, QueryStatus},
		modal, Spinner, Tag, Tags,
	},
	system::{
		core::SourceId,
		dnd5e::{
			components::{
				editor::{mutator_list, CollapsableCard},
				SharedCharacter,
			},
			data::{
				character::{ActionEffect, Character, Persistent},
				Condition,
			},
			DnD5e,
		},
	},
	utility::InputExt,
};
use yew::prelude::*;

#[function_component]
pub fn ConditionsCard() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let onclick = modal_dispatcher.callback(|_| {
		modal::Action::Open(modal::Props {
			centered: true,
			scrollable: true,
			root_classes: classes!("condition"),
			content: html! {<Modal />},
			..Default::default()
		})
	});
	let conditions = state
		.persistent()
		.conditions
		.iter()
		.map(|condition| {
			// TODO: Show which conditions are disabled in the card
			let _disabled = match &condition.criteria {
				None => false,
				Some(criteria) => criteria.evaluate(&state).is_ok(),
			};
			html! {
				<Tag>
					{condition.name.clone()}
				</Tag>
			}
		})
		.collect::<Vec<_>>();
	html! {
		<div class="card m-1" style="height: 85px;" {onclick}>
			<div class="card-body text-center" style="padding: 5px 5px;">
				<h6 class="card-title mb-1" style="font-size: 0.8rem;">{"Conditions"}</h6>
				<div class="d-flex justify-content-center pe-1" style="overflow: scroll; height: 53px;">
					{match conditions.is_empty() {
						true => html! { "None" },
						false => html! {<Tags> {conditions} </Tags>},
					}}
				</div>
			</div>
		</div>
	}
}

#[function_component]
fn Modal() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	let add_condition_section = {
		use crate::system::core::System;
		let conditions_handle = use_query_all_typed::<Condition>(QueryAllArgs {
			system: DnD5e::id().into(),
			auto_fetch: true,
			..Default::default()
		});
		let add_condition_by_id = use_typed_fetch_callback(
			"Add Condition".into(),
			state.new_dispatch(Box::new(
				move |condition: Condition, persistent: &mut Persistent, _: &Rc<Character>| {
					persistent.conditions.insert(condition);
					Some(ActionEffect::Recompile)
				},
			)),
		);
		let on_add_condition = Callback::from(move |evt: web_sys::Event| {
			let Some(value) = evt.select_value() else { return; };
			let Ok(source_id) = SourceId::from_str(&value) else { return; };
			add_condition_by_id.emit(source_id);
		});

		// TODO: This should use a regex-capable search bar, just like searching for an item in the inventory

		let content = match conditions_handle.status() {
			QueryStatus::Pending => html!(<Spinner />),
			QueryStatus::Empty | QueryStatus::Failed(_) => html! {
				<select class="form-select">
					<option value="" selected={true}>{"No conditions available"}</option>
				</select>
			},
			QueryStatus::Success(conditions) => {
				let options = conditions
					.iter()
					.map(|condition| {
						let Some(id) = &condition.id else { return html!(); };
						let id = id.unversioned();
						html! {
							<option
								value={id.to_string()}
								disabled={state.persistent().conditions.contains_id(&id)}
							>
								{condition.name.clone()}
							</option>
						}
					})
					.collect::<Vec<_>>();
				html! {
					<select class="form-select" onchange={on_add_condition}>
						<option value="" selected={true}>{"Pick a Condition..."}</option>
						{options}
					</select>
				}
			}
		};
		html! {
			<div class="input-group mb-3">
				<span class="input-group-text">{"Add a Condition"}</span>
				{content}
			</div>
		}
	};

	let on_remove_condition = Callback::from({
		let state = state.clone();
		move |key| {
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				persistent.conditions.remove(&key);
				Some(ActionEffect::Recompile)
			}));
		}
	});

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Conditions"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{add_condition_section}
			<div>
				{state.persistent().conditions.iter_keyed().map(|(key, condition)| {
					let on_remove = on_remove_condition.reform({
						let key = key.clone();
						move |_| key.clone()
					});
					// TODO: Show degrees in body of collapsable card
					html! {
						<CollapsableCard
							id={condition.name.clone()}
							header_content={{
								html! {<>
									<span>{condition.name.clone()}</span>
									<button
										type="button" class="btn-close ms-auto" aria-label="Close"
										onclick={on_remove}
									/>
								</>}
							}}
						>
							<div class="text-block">{condition.description.clone()}</div>
							{match &condition.criteria {
								None => html! {},
								Some(criteria) => html! {
									<div class="property">
										<strong>{"Criteria:"}</strong>
										<span>{criteria.description().unwrap_or_else(|| format!("criteria missing description"))}</span>
									</div>
								},
							}}
							{mutator_list(&condition.mutators, Some(&state))}
						</CollapsableCard>
					}
				}).collect::<Vec<_>>()}
			</div>
		</div>
	</>}
}
