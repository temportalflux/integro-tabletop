pub trait AsKdl {
	fn as_kdl(&self) -> NodeBuilder;
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

	pub fn append_typed(&mut self, ty: impl Into<kdl::KdlIdentifier>, mut node: NodeBuilder) {
		node.set_first_entry_ty(ty);
		*self += node;
	}

	pub fn push_entry(&mut self, entry: impl Into<kdl::KdlEntry>) {
		self.entries.push(entry.into());
	}

	pub fn push_entry_typed(
		&mut self,
		entry: impl Into<kdl::KdlEntry>,
		ty: impl Into<kdl::KdlIdentifier>,
	) {
		self.entries.push({
			let mut entry = entry.into();
			entry.set_ty(ty);
			entry
		});
	}

	pub fn with_entry(mut self, entry: impl Into<kdl::KdlEntry>) -> Self {
		self.push_entry(entry);
		self
	}

	pub fn with_entry_typed(
		mut self,
		entry: impl Into<kdl::KdlEntry>,
		ty: impl Into<kdl::KdlIdentifier>,
	) -> Self {
		self.push_entry_typed(entry, ty);
		self
	}

	pub fn push_child(&mut self, node: kdl::KdlNode) {
		self.children.push(node);
	}

	pub fn push_child_t(&mut self, name: impl Into<kdl::KdlIdentifier>, data: &impl AsKdl) {
		self.push_child(data.as_kdl().build(name))
	}

	pub fn push_child_opt(&mut self, node: kdl::KdlNode) {
		let has_children = node.children().map(|doc| !doc.is_empty()).unwrap_or(false);
		if !node.entries().is_empty() || has_children {
			self.push_child(node);
		}
	}

	pub fn push_child_opt_t(&mut self, name: impl Into<kdl::KdlIdentifier>, data: &impl AsKdl) {
		self.push_child_opt(data.as_kdl().build(name))
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

	pub fn push_child_entry_typed(
		&mut self,
		name: impl Into<kdl::KdlIdentifier>,
		ty: impl Into<kdl::KdlIdentifier>,
		entry: impl Into<kdl::KdlEntry>,
	) {
		self.push_child(
			Self::default()
				.with_entry_typed(entry.into(), ty)
				.build(name),
		);
	}
}

impl std::ops::AddAssign for NodeBuilder {
	fn add_assign(&mut self, mut rhs: Self) {
		self.entries.append(&mut rhs.entries);
		self.children.append(&mut rhs.children);
	}
}

macro_rules! impl_askdl_entry {
	($target:ty, $map:expr) => {
		impl AsKdl for $target {
			fn as_kdl(&self) -> NodeBuilder {
				NodeBuilder::default().with_entry(($map)(*self))
			}
		}
	};
}
impl_askdl_entry!(bool, |v| v);
impl_askdl_entry!(u8, |v| v as i64);
impl_askdl_entry!(i8, |v| v as i64);
impl_askdl_entry!(u16, |v| v as i64);
impl_askdl_entry!(i16, |v| v as i64);
impl_askdl_entry!(u32, |v| v as i64);
impl_askdl_entry!(i32, |v| v as i64);
impl_askdl_entry!(u64, |v| v as i64);
impl_askdl_entry!(i64, |v| v);
impl_askdl_entry!(u128, |v| v as i64);
impl_askdl_entry!(i128, |v| v as i64);
impl_askdl_entry!(usize, |v| v as i64);
impl_askdl_entry!(isize, |v| v as i64);
impl_askdl_entry!(f32, |v| v as f64);
impl_askdl_entry!(f64, |v| v);
impl AsKdl for String {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if !self.is_empty() {
			node.push_entry(self.clone());
		}
		node
	}
}
