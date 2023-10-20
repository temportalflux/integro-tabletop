use super::Restriction;
use std::collections::BTreeMap;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct CantripCapacity {
	pub class_name: String,
	pub level_map: BTreeMap<usize, usize>,
	pub restriction: Restriction,
}
