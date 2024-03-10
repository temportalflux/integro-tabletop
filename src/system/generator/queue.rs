use crate::system::{generator, generics};
use std::{
	collections::{BTreeMap, VecDeque},
	sync::Arc,
};

// Priority Queue of generators used to generate variants per generator in order of their urgency.
pub struct Queue {
	node_reg: Arc<generics::Registry>,
	generators_by_kind: BTreeMap<&'static str, VecDeque<generator::Generic>>,
}

impl Queue {
	pub fn new(node_reg: Arc<generics::Registry>) -> Self {
		Self {
			node_reg,
			generators_by_kind: BTreeMap::default(),
		}
	}

	pub fn len(&self) -> usize {
		self.generators_by_kind.iter().map(|(_kind, queue)| queue.len()).sum()
	}

	pub fn enqueue(&mut self, generator: generator::Generic) {
		if !self.generators_by_kind.contains_key(generator.kind()) {
			self.generators_by_kind.insert(generator.kind(), Default::default());
		}
		let Some(category) = self.generators_by_kind.get_mut(generator.kind()) else {
			return;
		};
		category.push_back(generator);
	}

	pub fn dequeue(&mut self) -> Option<generator::Generic> {
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
