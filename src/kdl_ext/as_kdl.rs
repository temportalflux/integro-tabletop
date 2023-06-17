pub trait AsKdl {
	fn as_kdl(&self) -> NodeBuilder;

	fn build_kdl(&self, name: impl Into<kdl::KdlIdentifier>) -> kdl::KdlNode {
		self.as_kdl().build(name)
	}
}

#[derive(Default)]
pub struct NodeBuilder {
	entries: Vec<kdl::KdlEntry>,
	children: Vec<kdl::KdlNode>,
}

impl NodeBuilder {
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

	pub fn set_first_entry_ty(&mut self, ty: impl Into<kdl::KdlIdentifier>) {
		if let Some(entry) = self.entries.get_mut(0) {
			entry.set_ty(ty);
		}
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

	pub fn push_child_t(&mut self, name: impl Into<kdl::KdlIdentifier>, data: &impl AsKdl) {
		self.push_child(data.build_kdl(name))
	}

	pub fn push_child_opt(&mut self, node: kdl::KdlNode) {
		let has_children = node.children().map(|doc| !doc.is_empty()).unwrap_or(false);
		if !node.entries().is_empty() || has_children {
			self.push_child(node);
		}
	}

	pub fn push_child_opt_t(&mut self, name: impl Into<kdl::KdlIdentifier>, data: &impl AsKdl) {
		self.push_child_opt(data.build_kdl(name))
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
		self.push_child(Self::default().with_entry(entry.into()).build(name));
	}
}

impl std::ops::AddAssign for NodeBuilder {
	fn add_assign(&mut self, mut rhs: Self) {
		self.entries.append(&mut rhs.entries);
		self.children.append(&mut rhs.children);
	}
}
