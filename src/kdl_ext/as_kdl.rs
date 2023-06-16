pub trait AsKdl {
	fn as_kdl(&self) -> NodeBuilder;

	fn build_kdl(&self, name: impl Into<kdl::KdlIdentifier>) -> kdl::KdlNode {
		self.as_kdl().build(name)
	}
}

pub struct NodeBuilder {
	entries: Vec<kdl::KdlEntry>,
	children: Vec<kdl::KdlNode>,
}

impl NodeBuilder {
	pub fn new() -> Self {
		Self {
			entries: Vec::new(),
			children: Vec::new(),
		}
	}

	pub fn build(self, name: impl Into<kdl::KdlIdentifier>) -> kdl::KdlNode {
		let Self {
			mut entries,
			mut children,
		} = self;
		let mut node = kdl::KdlNode::new(name);
		node.entries_mut().append(&mut entries);
		if !children.is_empty() {
			node.ensure_children().nodes_mut().append(&mut children);
		}
		node
	}

	pub fn push_entry(&mut self, entry: impl Into<kdl::KdlEntry>) {
		self.entries.push(entry.into());
	}

	pub fn with_entry(mut self, entry: impl Into<kdl::KdlEntry>) -> Self {
		self.push_entry(entry);
		self
	}

	pub fn push_child(&mut self, node: kdl::KdlNode) {
		self.children.push(node);
	}

	pub fn push_child_opt(&mut self, node: kdl::KdlNode) {
		let has_children = node.children().map(|doc| !doc.is_empty()).unwrap_or(false);
		if !node.entries().is_empty() || has_children {
			self.push_child(node);
		}
	}

	pub fn with_child(mut self, node: kdl::KdlNode) -> Self {
		self.push_child(node);
		self
	}

	pub fn push_child_entry(
		&mut self,
		name: impl Into<kdl::KdlIdentifier>,
		entry: impl Into<kdl::KdlEntry>,
	) {
		self.push_child(Self::new().with_entry(entry.into()).build(name));
	}
}
