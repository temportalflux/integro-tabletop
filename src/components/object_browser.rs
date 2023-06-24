use crate::{database::app::Criteria, kdl_ext::KDLNode, system::dnd5e::components::GeneralProp};
use std::{collections::HashMap, path::PathBuf, rc::Rc, str::FromStr};
use yew::prelude::*;

use super::modal;

#[function_component]
pub fn Provider(props: &html::ChildrenProps) -> Html {
	let registry = use_state(|| Registry::new());
	html! {
		<ContextProvider<Registry> context={(*registry).clone()}>
			{props.children.clone()}
		</ContextProvider<Registry>>
	}
}

#[derive(Clone)]
pub struct Registry(Rc<HashMap<String, Box<dyn ObjectBrowser>>>);
impl Registry {
	fn new() -> Self {
		let mut registry = HashMap::<_, Box<dyn ObjectBrowser>>::new();
		registry.insert(
			crate::system::dnd5e::data::Spell::id().to_owned(),
			Box::new(SpellBrowser),
		);
		Self(Rc::new(registry))
	}

	fn get(&self, key: impl AsRef<str>) -> Option<&Box<dyn ObjectBrowser>> {
		self.0.get(key.as_ref())
	}
}
impl PartialEq for Registry {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

trait ObjectBrowser {
	fn modal(&self, props: &ModalProps) -> Html;
}

pub fn open_modal(modal_dispatcher: &modal::Context, props: ModalProps) -> Callback<MouseEvent> {
	modal_dispatcher.callback({
		move |_| {
			let props = props.clone();
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				root_classes: classes!("browse", "objects"),
				content: html! {<Modal ..props />},
				..Default::default()
			})
		}
	})
}
#[derive(Clone, PartialEq, Properties)]
pub struct ModalProps {
	pub data_path: PathBuf,
	pub category: AttrValue,
	pub capacity: usize,
	pub criteria: Option<Criteria>,
}
#[function_component]
fn Modal(props: &ModalProps) -> Html {
	let registry = use_context::<Registry>().unwrap();
	if let Some(browser) = registry.get(props.category.as_str()) {
		return browser.modal(props);
	}
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Unsupported object category"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
	</>}
}

struct SpellBrowser;
impl ObjectBrowser for SpellBrowser {
	fn modal(&self, props: &ModalProps) -> Html {
		use crate::system::dnd5e::{
			components::panel::{AvailableSpellList, HeaderAddon},
			data::character::spellcasting,
			data::Spell,
		};

		let header_addon = HeaderAddon::from({
			let data_path = props.data_path.clone();
			let capacity = props.capacity;
			move |spell: &Spell| -> Html {
				html! {
					<ObjectSelectorEntryButton
						data_path={data_path.clone()}
						id={spell.id.unversioned()}
						{capacity}
					/>
				}
			}
		});

		// TODO: Somehow generate the spell entry for the feature's selector data
		let spell_entry = spellcasting::SpellEntry {
			ability: crate::system::dnd5e::data::Ability::Charisma,
			source: std::path::PathBuf::new(),
			classified_as: None,
			cast_via_slot: false,
			cast_via_uses: None,
			range: None,
			forced_rank: None,
		};

		html! {<>
			<div class="modal-header">
				<h1 class="modal-title fs-4">{"Browse Spells"}</h1>
				<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
			</div>
			<div class="modal-body spell-list">
				<AvailableSpellList
					{header_addon}
					criteria={props.criteria.clone()}
					entry={spell_entry}
				/>
			</div>
		</>}
	}
}

#[function_component]
pub fn ObjectSelectorList(props: &GeneralProp<std::path::PathBuf>) -> Html {
	use crate::{
		components::database::{use_query_entries, QueryStatus},
		system::{core::SourceId, dnd5e::components::CharacterHandle},
	};

	let state = use_context::<CharacterHandle>().unwrap();
	let fetched_entries = use_query_entries();
	use_effect_with_deps(
		{
			let data_path = props.value.clone();
			let fetched_entries = fetched_entries.clone();
			move |state: &CharacterHandle| {
				let Some(values) = state.get_selections_at(&data_path) else {
					fetched_entries.clear();
					return;
				};
				let mut ids = Vec::with_capacity(values.len());
				for value in values {
					let Ok(id) = SourceId::from_str(value.as_str()) else { continue; };
					ids.push(id.into_unversioned());
				}
				fetched_entries.run(ids);
			}
		},
		state.clone(),
	);

	match fetched_entries.status() {
		QueryStatus::Pending => html!(<crate::components::Spinner />),
		QueryStatus::Empty | QueryStatus::Failed(_) => html!("No selections"),
		QueryStatus::Success((ids, items)) => {
			html! {
				<ul class="mb-0">
					{ids.iter().filter_map(|id| items.get(id)).map(|entry| html! {
						<li>{entry.name().unwrap_or("Unknown")}</li>
					}).collect::<Vec<_>>()}
				</ul>
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ObjectSelectorEntryButtonProps {
	data_path: std::path::PathBuf,
	id: crate::system::core::SourceId,
	capacity: usize,
}
#[function_component]
fn ObjectSelectorEntryButton(props: &ObjectSelectorEntryButtonProps) -> Html {
	let state = use_context::<crate::system::dnd5e::components::CharacterHandle>().unwrap();

	let props_id = props.id.to_string();
	let mut selected_idx = None;
	let mut can_select_more = props.capacity > 0;

	if let Some(entries) = state.get_selections_at(&props.data_path) {
		can_select_more = entries.len() < props.capacity;
		for (idx, id_str) in entries.iter().enumerate() {
			if id_str.as_str() == props_id.as_str() {
				selected_idx = Some(idx);
				break;
			}
		}
	}

	let is_selected = selected_idx.is_some();
	let onclick = state.new_dispatch({
		let data_path = props.data_path.clone();
		move |evt: MouseEvent, persistent| {
			evt.stop_propagation();
			match selected_idx {
				None => {
					persistent.insert_selection(&data_path, props_id.clone());
				}
				Some(idx) => {
					persistent.remove_selection(&data_path, idx);
				}
			}
			// recompile required because mutators which have object selections
			// probably need to use those selections to affect derived data
			// (e.g. spellcasting add_prepared)
			crate::page::characters::sheet::MutatorImpact::Recompile
		}
	});

	let mut classes = classes!("btn", "btn-xs");
	let disabled = !is_selected && !can_select_more;
	classes.push(match is_selected {
		true => "btn-outline-theme",
		false => match can_select_more {
			true => "btn-theme",
			false => "btn-outline-secondary",
		},
	});

	html! {
		<button
			type="button" class={classes}
			style={"margin-left: auto; width: 85px;"}
			{disabled} {onclick}
		>
			{match is_selected {
				true => "Selected",
				false => "Select",
			}}
		</button>
	}
}
