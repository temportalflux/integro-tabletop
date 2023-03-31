use std::collections::HashSet;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Description {
	pub name: String,
	pub pronouns: HashSet<String>,
	pub custom_pronouns: String,
}
