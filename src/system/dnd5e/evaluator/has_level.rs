use crate::{kdl_ext::NodeContext, system::dnd5e::data::character::Character};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasLevel {
	class_name: Option<String>,
	minimum: usize,
}

crate::impl_trait_eq!(HasLevel);
kdlize::impl_kdl_node!(HasLevel, "level");

impl crate::system::Evaluator for HasLevel {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(match &self.class_name {
			None => format!("level {}", self.minimum),
			Some(class_name) => format!("{class_name} level {}", self.minimum),
		})
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		let class_name = self.class_name.as_ref().map(String::as_str);
		let level = character.level(class_name);
		if level >= self.minimum {
			return Ok(());
		}
		Err(format!("not at least {}", self.description().expect("missing description")))
	}
}

impl FromKdl<NodeContext> for HasLevel {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt("class")?.map(str::to_owned);
		let minimum = node.get_i64_req("min")? as usize;
		Ok(Self { class_name, minimum })
	}
}

impl AsKdl for HasLevel {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(("class", self.class_name.clone()));
		node.entry(("min", self.minimum as i64));
		node
	}
}
