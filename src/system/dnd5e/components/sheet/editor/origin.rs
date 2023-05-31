use crate::{
	components::{
		database::{use_query_all_typed, use_typed_fetch_callback, QueryAllArgs, QueryStatus},
		modal, Spinner,
	},
	database::app::Criteria,
	system::{
		core::{SourceId, System},
		dnd5e::{
			components::{GeneralProp, SharedCharacter},
			data::{
				bundle::BundleRequirement,
				character::{
					spellcasting::{SpellEntry, SpellFilter},
					ActionEffect, Persistent,
				},
				description, Bundle, Feature, Spell,
			},
			DnD5e,
		},
	},
	utility::{
		web_ext::{self, CallbackExt, CallbackOptExt},
		GenericMutator, InputExt, SelectorMeta, SelectorOptions,
	},
};
use convert_case::{Case, Casing};
use multimap::MultiMap;
use std::{
	collections::{HashSet},
	str::FromStr,
	sync::Arc,
};
use yew::prelude::*;

static HELP_TEXT: &'static str = "Lineages and Upbingings are a replacement for races. \
They offer an expanded set of options around traits and features granted from \
the parents and community your character comes from.";

#[function_component]
pub fn OriginTab() -> Html {
	let use_lineages = use_state_eq(|| true);

	let toggle_lineages = web_ext::callback()
		.map(|evt: web_sys::Event| evt.input_checked())
		.on_some({
			let use_lineages = use_lineages.clone();
			move |checked| use_lineages.set(checked)
		});
	let lineages_switch = html! {
		<div class="form-check form-switch m-2">
			<label for="useLineages" class="form-check-label">{"Use Lineages & Upbringings"}</label>
			<input  id="useLineages" class="form-check-input"
				type="checkbox" role="switch"
				checked={*use_lineages}
				onchange={toggle_lineages}
			/>
			<div id="useLineagesHelpBlock" class="form-text">{HELP_TEXT}</div>
		</div>
	};

	html! {<>
		{lineages_switch}
		<CharacterContent />
		<CategoryBrowser use_lineages={*use_lineages} />
	</>}
}

#[function_component]
fn CharacterContent() -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

	// Map of things required (Bundle Category, Name) to that which requires them (Bundle Category, Name)
	let requirements = {
		let mut map = MultiMap::new();
		for bundle in &state.persistent().bundles {
			for req in &bundle.requirements {
				if let BundleRequirement::Bundle { category, name } = req {
					map.insert((category, name), (&bundle.category, &bundle.name));
				}
			}
		}
		map
	};

	if state.persistent().bundles.is_empty() {
		return html! {};
	}
	html! {<>
		<div class="accordion mt-2 mb-4" id="selected-content">
			{state.persistent().bundles.iter().enumerate().map(|(idx, bundle)| {
				selected_bundle(&state, idx, bundle, &requirements)
			}).collect::<Vec<_>>()}
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct CategoryBrowserProps {
	use_lineages: bool,
}

#[function_component]
fn CategoryBrowser(CategoryBrowserProps { use_lineages: _ }: &CategoryBrowserProps) -> Html {
	let selected_category = use_state(|| None::<AttrValue>);
	let query_bundles = use_query_all_typed::<Bundle>(QueryAllArgs {
		auto_fetch: false,
		system: DnD5e::id().into(),
		criteria: Some(Arc::new({
			let selected = selected_category.clone();
			move || {
				let category = selected.as_ref().map(AttrValue::as_str).unwrap().to_owned();
				let matches_category = Criteria::Exact(category.into());
				Some(Criteria::ContainsProperty("category".into(), matches_category.into()).into())
			}
		})),
		adjust_listings: Some(Arc::new(|mut bundles| {
			bundles.sort_by(|a, b| a.name.cmp(&b.name));
			bundles
		})),
		..Default::default()
	});
	// Query for bundles when the category changes
	use_effect_with_deps(
		{
			let query_bundles = query_bundles.clone();
			move |category: &UseStateHandle<Option<AttrValue>>| {
				if category.is_some() {
					query_bundles.run();
				}
			}
		},
		selected_category.clone(),
	);

	let options = vec![
		"Race",
		"Race Variant",
		"Lineage",
		"Upbringing",
		"Background",
		"Feat",
	];
	html! {<>
		<CategoryPicker
			value={(*selected_category).clone()}
			options={options}
			on_change={Callback::from({
				let selected_category = selected_category.clone();
				move |value| selected_category.set(value)
			})}
		/>
		{match query_bundles.status() {
			QueryStatus::Pending => html!(<Spinner />),
			QueryStatus::Empty | QueryStatus::Failed(_) => html!("No available bundles"),
			QueryStatus::Success(bundles) => html! {
				<div class="accordion my-2" id="all-entries">
					{bundles.iter().map(|bundle| {
						html!(<AvailableBundle value={bundle.clone()} />)
					}).collect::<Vec<_>>()}
				</div>
			},
		}}
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct CategoryPickerProps {
	options: Vec<&'static str>,
	value: Option<AttrValue>,
	on_change: Callback<Option<AttrValue>>,
}
#[function_component]
fn CategoryPicker(
	CategoryPickerProps {
		options,
		value,
		on_change,
	}: &CategoryPickerProps,
) -> Html {
	let on_selection_changed = Callback::from({
		let on_change = on_change.clone();
		move |evt: web_sys::Event| {
			let Some(value) = evt.select_value() else { return; };
			on_change.emit((!value.is_empty()).then_some(value.into()));
		}
	});
	let close_btn = match value.is_some() {
		true => html! {
			<button type="button"
				class="btn btn-outline-warning"
				onclick={on_change.reform(|_| None)}
			>
				{"Close"}
			</button>
		},
		false => html! {},
	};
	html! {
		<div class="input-group">
			<span class="input-group-text">{"Browse Categories"}</span>
			<select class="form-select" onchange={on_selection_changed} disabled={value.is_some()}>
				<option
					value=""
					selected={value.is_none()}
				>{"Select a category..."}</option>
				{options.iter().map(|item| html! {
					<option
						value={item.clone()}
						selected={value.as_ref().map(AttrValue::as_str) == Some(*item)}
					>{item.clone()}</option>
				}).collect::<Vec<_>>()}
			</select>
			{close_btn}
		</div>
	}
}

fn selected_bundle(
	state: &SharedCharacter,
	idx: usize,
	bundle: &Bundle,
	all_dependents: &MultiMap<(&String, &String), (&String, &String)>,
) -> Html {
	let dependents = match all_dependents.get_vec(&(&bundle.category, &bundle.name)) {
		None => None,
		Some(reqs) => Some(
			reqs.iter()
				.map(|(category, name)| format!("{category}: {name}"))
				.collect::<Vec<_>>()
				.join(", "),
		),
	};

	let reqs_desc = {
		let reqs = bundle
			.requirements
			.iter()
			.map(|req| match req {
				BundleRequirement::Bundle { category, name } => format!("{category}: {name}"),
				BundleRequirement::Ability(ability, score) => {
					format!("{} >= {score}", ability.long_name())
				}
			})
			.collect::<Vec<_>>();
		(!reqs.is_empty()).then(|| format!(" (requires: [{}])", reqs.join(", ")))
	};
	let title = reqs_desc
		.map(|desc| format!("{}{desc}", bundle.name))
		.unwrap_or_else(|| bundle.name.clone());

	html! {
		<ContentItem
			id={format!("{}-{}-{}", bundle.category, idx, bundle.name.to_case(Case::Kebab))}
			name={format!("{}: {}", bundle.category, title)}
			kind={ContentItemKind::Remove {
				disable_selection: dependents.map(|desc| format!("Cannot remove, depended on by: {desc}").into()),
			}}
			on_click={state.new_dispatch(move |_, persistent, _| {
				persistent.bundles.remove(idx);
				Some(ActionEffect::Recompile) // TODO: Only do this when returning to sheet view
			})}
		>
			<div class="text-block">
				<DescriptionSection section={bundle.description.clone()} show_selectors={true} />
			</div>
			{mutator_list(&bundle.mutators, Some(state))}
			{bundle.features.iter().map(|f| feature(f,  Some(state))).collect::<Vec<_>>()}
		</ContentItem>
	}
}

#[function_component]
fn AvailableBundle(GeneralProp { value: bundle }: &GeneralProp<Bundle>) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let on_select = use_typed_fetch_callback(
		"Add Bundle".into(),
		state.new_dispatch(|bundle: Bundle, persistent, _| {
			persistent.bundles.push(bundle);
			Some(ActionEffect::Recompile) // TODO: Only do this when returning to sheet view
		}),
	);

	let amount_selected = state
		.persistent()
		.bundles
		.iter()
		.filter(|selected| selected.id == bundle.id)
		.count();
	let mut title = bundle.name.clone();
	let mut requirements_met = true;
	if !bundle.requirements.is_empty() {
		let reqs = bundle
			.requirements
			.iter()
			.map(|req| match req {
				BundleRequirement::Bundle { category, name } => format!("{category}: {name}"),
				BundleRequirement::Ability(ability, score) => {
					format!("{} >= {score}", ability.long_name())
				}
			})
			.collect::<Vec<_>>();
		title = format!("{title} (requires: [{}])", reqs.join(", "));
		for req in &bundle.requirements {
			match req {
				BundleRequirement::Bundle { category, name } => {
					let mut iter_bundles = state.persistent().bundles.iter();
					let passed = iter_bundles
						.find(|selected| &selected.category == category && &selected.name == name)
						.is_some();
					if !passed {
						requirements_met = false;
						break;
					}
				}
				BundleRequirement::Ability(ability, score) => {
					if state.ability_scores().get(*ability).score().0 < *score {
						requirements_met = false;
						break;
					}
				}
			}
		}
	}
	html! {
		<ContentItem
			parent_collapse={"#all-entries"}
			id={bundle.name.clone()}
			name={title}
			kind={ContentItemKind::Add {
				amount_selected,
				selection_limit: bundle.limit,
				disable_selection: (!requirements_met).then(|| "Requirements not met".into()),
			}}
			on_click={on_select.reform({
				let source_id = bundle.id.unversioned();
				move |_| source_id.clone()
			})}
		>
			<div class="text-block">
				<DescriptionSection section={bundle.description.clone()} show_selectors={false} />
			</div>
			{mutator_list(&bundle.mutators, None::<&SharedCharacter>)}
			{bundle.features.iter().map(|f| feature(f,  None)).collect::<Vec<_>>()}
		</ContentItem>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ContentItemProps {
	#[prop_or_default]
	parent_collapse: Option<AttrValue>,
	id: AttrValue,
	name: AttrValue,
	kind: ContentItemKind,
	children: Children,
	on_click: Callback<()>,
}
#[derive(Clone, PartialEq)]
enum ContentItemKind {
	Add {
		amount_selected: usize,
		selection_limit: usize,
		disable_selection: Option<AttrValue>,
	},
	Remove {
		disable_selection: Option<AttrValue>,
	},
}
#[function_component]
fn ContentItem(
	ContentItemProps {
		parent_collapse,
		id,
		name,
		kind,
		children,
		on_click,
	}: &ContentItemProps,
) -> Html {
	let disabled_btn = |text: Html| {
		html! {
			<button type="button" class="btn btn-outline-secondary my-1 w-100" disabled={true}>
				{text}
			</button>
		}
	};
	let slot_buttons = match kind {
		ContentItemKind::Add {
			amount_selected,
			selection_limit,
			disable_selection,
		} => {
			let select_btn = |text: Html| {
				html! {
					<button type="button" class="btn btn-outline-success my-1 w-100" onclick={on_click.reform(|_| ())}>
						{text}
					</button>
				}
			};

			match (disable_selection, *amount_selected, *selection_limit) {
				(Some(reason), _, _) => disabled_btn(html! {{reason.clone()}}),
				// Slot is empty, and this option is not-yet used
				(_, 0, _) => select_btn(html! {{"Add"}}),
				// Slot is empty, this option is in another slot, but it can be used again
				(_, count, limit) if count < limit => {
					select_btn(html! {{format!("Add Again ({} / {})", count, limit)}})
				}
				// option already selected for another slot, and cannot be selected again
				(_, count, limit) if count >= limit => {
					disabled_btn(html! {{format!("Cannot Add Again ({} / {})", count, limit)}})
				}
				_ => html! {},
			}
		}
		ContentItemKind::Remove { disable_selection } => match disable_selection {
			Some(reason) => disabled_btn(html! {{reason.clone()}}),
			None => html! {
				<button type="button" class="btn btn-outline-danger my-1 w-100" onclick={on_click.reform(|_| ())}>
					{"Remove"}
				</button>
			},
		},
	};

	html! {
		<div class="accordion-item">
			<h2 class="accordion-header">
				<button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target={format!("#{id}")}>
					{name.clone()}
				</button>
			</h2>
			<div id={id.clone()} class="accordion-collapse collapse" data-bs-parent={parent_collapse.clone()}>
				<div class="accordion-body">
					<div>{slot_buttons}</div>
					{children.clone()}
				</div>
			</div>
		</div>
	}
}

pub fn feature(value: &Feature, state: Option<&SharedCharacter>) -> Html {
	let desc = match (state, value.description.clone()) {
		(Some(state), desc) => desc.evaluate(state),
		(None, desc) => desc,
	};

	let criteria = {
		let criteria = value.criteria.as_ref();
		let desc = criteria.map(|eval| eval.description()).flatten();
		desc.map(|text| {
			html! {
				<div class="property">
					<strong>{"Criteria"}</strong>
					{text}
				</div>
			}
		})
	};

	html! {
		<div class="my-2">
			<h5>{value.name.clone()}</h5>
			{description(&desc, false)}
			{mutator_list(&value.mutators, state)}
			{criteria.unwrap_or_default()}
		</div>
	}
}

// TODO: Unify with DescriptionSection
pub fn description(info: &description::Info, prefer_short: bool) -> Html {
	if prefer_short {
		if let Some(desc) = &info.short {
			return html! { <div class="text-block">{desc}</div> };
		}
	}
	let sections = info
		.sections
		.iter()
		.map(|section| {
			html! { <DescriptionSection section={section.clone()} show_selectors={false} /> }
		})
		.collect::<Vec<_>>();
	html! {
		<div>
			{sections}
		</div>
	}
}

pub fn mutator_list<T: 'static>(
	list: &Vec<GenericMutator<T>>,
	state: Option<&impl AsRef<T>>,
) -> Html {
	let mutators = list
		.iter()
		.filter_map(|value| mutator(value, state))
		.collect::<Vec<_>>();
	html! {<>{mutators}</>}
}

fn mutator<T: 'static>(value: &GenericMutator<T>, state: Option<&impl AsRef<T>>) -> Option<Html> {
	let target = state.map(|t| t.as_ref());
	let section = value.description(target);
	Some(html! { <DescriptionSection {section} show_selectors={state.is_some()} /> })
}

#[derive(Clone, PartialEq, Properties)]
pub struct SectionProps {
	pub section: description::Section,
	pub show_selectors: bool,
}
#[function_component]
pub fn DescriptionSection(
	SectionProps {
		section,
		show_selectors,
	}: &SectionProps,
) -> Html {
	let name = match &section.title {
		None => None,
		Some(title) => Some(html! {<strong>{title.clone()}{". "}</strong>}),
	};

	let content = match &section.content {
		description::SectionContent::Body(text) => {
			html! {
				<div class="text-block">
					{text}
				</div>
			}
		}
		description::SectionContent::Selectors(selectors) => show_selectors
			.then(|| {
				if !selectors.errors().is_empty() {
					log::warn!(target: "utility", "Section has empty data path: {section:?}");
				}
				let iter_selectors = selectors.as_vec().iter();
				let fields = iter_selectors
					.map(|meta| html! { <SelectorField meta={meta.clone()} /> })
					.collect::<Vec<_>>();
				html! {
					<div>
						{fields}
					</div>
				}
			})
			.unwrap_or_default(),
		description::SectionContent::Table {
			column_count: _,
			headers,
			rows,
		} => {
			html! {
				<table class="table table-compact table-striped m-0">
					<thead>
						<tr class="text-center" style="color: var(--bs-heading-color);">
							{match headers.as_ref() {
								None => html!(),
								Some(headers) => html! {<>
									{headers.iter().map(|s| html! {
										<th scope="col">{s}</th>
									}).collect::<Vec<_>>()}
								</>},
							}}
						</tr>
					</thead>
					<tbody>
						{rows.iter().map(|cols| {
							html! { <tr>
								{cols.iter().map(|s| html! {
									<td>{s}</td>
								}).collect::<Vec<_>>()}
							</tr> }
						}).collect::<Vec<_>>()}
					</tbody>
				</table>
			}
		}
	};

	let children = (!section.children.is_empty()).then(|| {
		html! {
			<div class="ms-2">
				{section.children.iter().map(|section| html! {
					<DescriptionSection section={section.clone()} show_selectors={*show_selectors} />
				}).collect::<Vec<_>>()}
			</div>
		}
	});

	html! {
		<div>
			{name.unwrap_or_default()}
			{content}
			{children}
		</div>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct SelectorFieldProps {
	meta: SelectorMeta,
}
#[function_component]
fn SelectorField(
	SelectorFieldProps {
		meta: SelectorMeta {
			name,
			data_path,
			options,
		},
	}: &SelectorFieldProps,
) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let modal_dispatcher = use_context::<modal::Context>().unwrap();
	let value = state.get_first_selection(data_path);

	let save_value = Callback::from({
		let data_path = data_path.clone();
		let state = state.clone();
		move |value| {
			let data_path = data_path.clone();
			state.dispatch(Box::new(move |persistent: &mut Persistent, _| {
				match value {
					None => {
						persistent.selected_values.remove(&data_path);
					}
					Some(value) => {
						persistent.selected_values.set(&data_path, value);
					}
				}
				Some(ActionEffect::Recompile) // TODO: Only do this when returning to sheet view
			}));
		}
	});

	let classes = classes!("my-2", "selector");
	let missing_value = value
		.is_none()
		.then(|| classes!("missing-value"))
		.unwrap_or_default();
	let inner = match options {
		SelectorOptions::Any => {
			let onchange = Callback::from({
				let save_value = save_value.clone();
				move |evt: web_sys::Event| {
					let Some(value) = evt.input_value() else { return; };
					save_value.emit((!value.is_empty()).then_some(value.into()));
				}
			});
			html! {<input
				class="form-control" type="text"
				value={value.cloned().unwrap_or_default()}
				{onchange}
			/>}
		}
		SelectorOptions::AnyOf {
			options: valid_values,
			cannot_match,
			amount: _, // TODO: Display a different UI if amount > 1
		} => {
			let onchange = Callback::from({
				let save_value = save_value.clone();
				move |evt: web_sys::Event| {
					let Some(value) = evt.select_value() else { return; };
					save_value.emit((!value.is_empty()).then_some(value.into()));
				}
			});
			let invalid_values = match cannot_match {
				None => HashSet::new(),
				Some(selection_paths) => {
					let mut values = HashSet::new();
					for path in selection_paths {
						if let Some(value) = state.get_first_selection(path) {
							values.insert(value);
						}
					}
					values
				}
			};
			html! {
				<select class="form-select" {onchange}>
					<option
						value=""
						selected={value.is_none()}
					>{"Select a value..."}</option>
					{valid_values.iter().map(|item| {
						html! {
							<option
								value={item.clone()}
								selected={value == Some(item)}
								disabled={invalid_values.contains(item)}
							>
								{item.clone()}
							</option>
						}
					}).collect::<Vec<_>>()}
				</select>
			}
		}
		SelectorOptions::Object {
			category,
			count: capacity,
			spell_filter,
		} => {
			let browse = modal_dispatcher.callback({
				let data_path = data_path.clone();
				let category: AttrValue = category.clone().into();
				let capacity = *capacity;
				let spell_filter = spell_filter.clone();
				move |_| {
					let data_path = data_path.clone();
					let category = category.clone();
					let filter = spell_filter.clone();
					modal::Action::Open(modal::Props {
						centered: true,
						scrollable: true,
						root_classes: classes!("browse", "objects"),
						content: html! {<ModalObjectBrowser {data_path} {category} {capacity} {filter} />},
						..Default::default()
					})
				}
			});
			let btn_classes = classes!("btn", "btn-outline-theme", "btn-xs", missing_value);
			let selection_count = state
				.get_selections_at(data_path)
				.map(|list| list.len())
				.unwrap_or_default();
			return html! {
				<div class={classes}>
					<h6>{name.clone()}</h6>
					<button type="button" class={btn_classes} onclick={browse}>
						{format!("Browse ({}/{capacity} selected)", selection_count)}
					</button>
					<ObjectSelectorList value={data_path.clone()} />
				</div>
			};
		}
	};
	html! {
		<div class={classes!("input-group", classes, missing_value)} style="max-width: 300px;">
			<span class="input-group-text">{name.clone()}</span>
			{inner}
		</div>
	}
}

#[function_component]
fn ObjectSelectorList(props: &GeneralProp<std::path::PathBuf>) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();
	let system = use_context::<UseStateHandle<DnD5e>>().unwrap();
	let mut entries = Vec::new();
	if let Some(selected_values) = state.get_selections_at(&props.value) {
		for id_str in selected_values {
			let Ok(id) = SourceId::from_str(id_str) else { continue; };
			// TODO: Get this from the database, not DnD5e in-memory
			let Some(spell) = system.spells.get(&id) else { continue; };
			entries.push(html! {
				<li>
					{&spell.name}
				</li>
			})
		}
	}
	html! {
		<ul class="mb-0">
			{entries}
		</ul>
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ModalObjectBrowserProps {
	data_path: std::path::PathBuf,
	category: AttrValue,
	capacity: usize,
	filter: Option<SpellFilter>,
}
#[function_component]
fn ModalObjectBrowser(props: &ModalObjectBrowserProps) -> Html {
	use crate::system::dnd5e::components::panel::{AvailableSpellList, HeaderAddon};

	// TODO: This modal should query the database and check for objects with provided category that meet the provided criteria,
	// checking against database metadata instead of the actual parsed objects.
	if props.category.as_str() != "spell" {
		return html! {<>
			<div class="modal-header">
				<h1 class="modal-title fs-4">{"Unsupported object category"}</h1>
				<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
			</div>
		</>};
	}

	let header_addon = HeaderAddon::from({
		let data_path = props.data_path.clone();
		let capacity = props.capacity;
		move |spell: &Spell| -> Html {
			html! {
				<ObjectSelectorEntryButton
					data_path={data_path.clone()}
					id={spell.id.clone()}
					{capacity}
				/>
			}
		}
	});
	// TODO: Somehow generate the spell entry for the feature's selector data
	let spell_entry = SpellEntry {
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
				filter={props.filter.clone().unwrap_or_default()}
				entry={spell_entry}
			/>
		</div>
	</>}
}

#[derive(Clone, PartialEq, Properties)]
struct ObjectSelectorEntryButtonProps {
	data_path: std::path::PathBuf,
	id: SourceId,
	capacity: usize,
}
#[function_component]
fn ObjectSelectorEntryButton(props: &ObjectSelectorEntryButtonProps) -> Html {
	let state = use_context::<SharedCharacter>().unwrap();

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
		move |evt: MouseEvent, persistent, _| {
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
			Some(ActionEffect::Recompile)
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
