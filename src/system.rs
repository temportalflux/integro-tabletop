use crate::GeneralError;
use std::{collections::HashMap, sync::Arc};
use yew::prelude::*;

pub mod core;
pub mod dnd5e;

#[derive(Clone)]
pub struct Depot(Arc<HashMap<&'static str, Registration>>);
impl PartialEq for Depot {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}
pub struct Registration {
	pub component: self::dnd5e::ComponentRegistry,
	pub node: Arc<self::core::NodeRegistry>,
}
impl Depot {
	fn new() -> Self {
		use self::core::System;
		let mut systems = HashMap::new();
		systems.insert(
			dnd5e::DnD5e::id(),
			Registration {
				component: dnd5e::component_registry(),
				node: dnd5e::node_registry().into(),
			},
		);
		Self(Arc::new(systems))
	}

	pub fn get_sys<T: self::core::System>(&self) -> Option<&Registration> {
		self.get(T::id())
	}

	pub fn get(&self, system_id: &str) -> Option<&Registration> {
		self.0.get(system_id)
	}
}
impl Registration {
	pub fn node(&self) -> Arc<self::core::NodeRegistry> {
		self.node.clone()
	}

	pub fn parse_metadata(
		&self,
		node: &kdl::KdlNode,
		source_id: &self::core::SourceId,
	) -> anyhow::Result<serde_json::Value> {
		use anyhow::Context;
		let category = node.name().value().to_owned();
		let Some(comp_factory) = self.component.get_factory(&category).cloned() else {
			return Err(GeneralError(format!("No component registered with id {category:?}")).into());
		};
		let ctx = crate::kdl_ext::NodeContext::new(Arc::new(source_id.clone()), self.node.clone());
		let metadata = comp_factory
			.metadata_from_kdl(node, &ctx)
			.with_context(|| format!("Failed to parse {:?}", source_id.to_string()))?;
		match metadata {
			serde_json::Value::Null => Ok(serde_json::Value::Null),
			serde_json::Value::Object(mut metadata) => {
				if let Some(module_id) = &source_id.module {
					metadata.insert("module".into(), serde_json::json!(module_id.to_string()));
				}
				Ok(serde_json::Value::Object(metadata))
			}
			other => {
				return Err(GeneralError(format!(
					"Metadata must be a map, but {} returned {:?}.",
					source_id.to_string(),
					other
				))
				.into());
			}
		}
	}
}

#[function_component]
pub fn Provider(props: &html::ChildrenProps) -> Html {
	let depot = use_state(|| Depot::new());
	html! {
		<ContextProvider<Depot> context={(*depot).clone()}>
			{props.children.clone()}
		</ContextProvider<Depot>>
	}
}
