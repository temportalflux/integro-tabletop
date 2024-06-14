use std::path::PathBuf;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct NumbericalBonusList(Vec<(i64, Option<String>, PathBuf)>);

impl NumbericalBonusList {
	pub fn push(&mut self, bonus: i64, context: Option<String>, source: PathBuf) {
		self.0.push((bonus, context, source));
	}

	pub fn iter(&self) -> impl Iterator<Item = &(i64, Option<String>, PathBuf)> {
		self.0.iter()
	}
}
