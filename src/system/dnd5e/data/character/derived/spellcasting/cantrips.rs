use std::collections::BTreeMap;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct CantripCapacity {
	pub class_name: String,
	pub level_map: BTreeMap<usize, usize>,
	pub restriction: Restriction,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Restriction {
	pub tags: Vec<String>,
}
