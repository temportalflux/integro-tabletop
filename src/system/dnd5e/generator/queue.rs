use crate::{system::core::NodeRegistry, utility::GenericGenerator};
use std::{
	collections::{BTreeMap, VecDeque},
	sync::Arc,
};

// Priority Queue of generators used to generate variants per generator in order of their urgency.
pub struct Queue {
	node_reg: Arc<NodeRegistry>,
	generators_by_kind: BTreeMap<&'static str, VecDeque<GenericGenerator>>,
}

impl Queue {
	pub fn new(node_reg: Arc<NodeRegistry>) -> Self {
		Self {
			node_reg,
			generators_by_kind: BTreeMap::default(),
		}
	}

	pub fn enqueue(&mut self, generator: GenericGenerator) {
		let Some(category) = self.generators_by_kind.get_mut(generator.kind()) else {
			return;
		};
		category.push_back(generator);
	}

	pub fn dequeue(&mut self) -> Option<GenericGenerator> {
		for generator_kind in self.node_reg.get_generator_order() {
			let Some(generators) = self.generators_by_kind.get_mut(*generator_kind) else {
				continue;
			};
			if let Some(generator) = generators.pop_front() {
				return Some(generator);
			}
		}
		None
	}
}
