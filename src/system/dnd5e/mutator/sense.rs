use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::data::{bounded::BoundValue, character::Character, description},
	system::Mutator,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub struct Sense {
	pub name: String,
	pub argument: BoundValue,
}

crate::impl_trait_eq!(Sense);
kdlize::impl_kdl_node!(Sense, "sense");

impl Mutator for Sense {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		let name = &self.name;
		description::Section {
			content: match &self.argument {
				BoundValue::Minimum(value) => format!("You have {name} for at least {value} feet."),
				BoundValue::Base(value) => format!("You have {name} for at least {value} feet."),
				BoundValue::Additive(value) => {
					format!("If you have {name} from another source, your {name} increases by {value} feet.")
				}
				BoundValue::Subtract(value) => {
					format!("If you have {name}, it decreases by {value} feet.")
				}
			}
			.into(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		stats
			.senses_mut()
			.insert(self.name.clone(), self.argument.clone(), parent);
	}
}

impl FromKdl<NodeContext> for Sense {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let name = node.next_str_req()?.to_owned();
		let argument = BoundValue::from_kdl(node)?;
		Ok(Self { name, argument })
	}
}

impl AsKdl for Sense {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(self.name.clone());
		node += self.argument.as_kdl();
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(Sense);

		#[test]
		fn minimum() -> anyhow::Result<()> {
			let doc = "mutator \"sense\" \"Darkvision\" (Minimum)60";
			let data = Sense {
				name: "Darkvision".into(),
				argument: BoundValue::Minimum(60),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn additive() -> anyhow::Result<()> {
			let doc = "mutator \"sense\" \"Darkvision\" (Additive)60";
			let data = Sense {
				name: "Darkvision".into(),
				argument: BoundValue::Additive(60),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			bounded::BoundKind,
			character::{Character, Persistent},
			Bundle,
		};

		fn character(mutators: Vec<(&'static str, Sense)>) -> Character {
			Character::from(Persistent {
				bundles: mutators
					.into_iter()
					.map(|(name, mutator)| {
						Bundle {
							name: name.into(),
							mutators: vec![mutator.into()],
							..Default::default()
						}
						.into()
					})
					.collect(),
				..Default::default()
			})
		}

		#[test]
		fn minimum_single() {
			let character = character(vec![(
				"TestFeature",
				Sense {
					name: "Darkvision".into(),
					argument: BoundValue::Minimum(60),
				},
			)]);
			let sense = character.senses().get("Darkvision").unwrap();
			let expected = [(BoundKind::Minimum, [("TestFeature".into(), 60)].into())].into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 60);
		}

		#[test]
		fn minimum_multiple() {
			let character = character(vec![
				(
					"SenseB",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Minimum(60),
					},
				),
				(
					"SenseA",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Minimum(40),
					},
				),
			]);
			let sense = character.senses().get("Darkvision").unwrap();
			let expected = [(
				BoundKind::Minimum,
				[("SenseA".into(), 40), ("SenseB".into(), 60)].into(),
			)]
			.into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 60);
		}

		#[test]
		fn single_additive() {
			let character = character(vec![(
				"TestFeature",
				Sense {
					name: "Darkvision".into(),
					argument: BoundValue::Additive(20),
				},
			)]);
			let sense = character.senses().get("Darkvision").unwrap();
			let expected = [(BoundKind::Additive, [("TestFeature".into(), 20)].into())].into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 20);
		}

		#[test]
		fn minimum_gt_additive() {
			let character = character(vec![
				(
					"A",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Minimum(60),
					},
				),
				(
					"B",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Additive(40),
					},
				),
			]);
			let sense = character.senses().get("Darkvision").unwrap();
			let expected = [
				(BoundKind::Minimum, [("A".into(), 60)].into()),
				(BoundKind::Additive, [("B".into(), 40)].into()),
			]
			.into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 60);
		}

		#[test]
		fn minimum_lt_additive() {
			let character = character(vec![
				(
					"A",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Minimum(60),
					},
				),
				(
					"B",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Additive(40),
					},
				),
				(
					"C",
					Sense {
						name: "Darkvision".into(),
						argument: BoundValue::Additive(30),
					},
				),
			]);
			let sense = character.senses().get("Darkvision").unwrap();
			let expected = [
				(BoundKind::Minimum, [("A".into(), 60)].into()),
				(BoundKind::Additive, [("B".into(), 40), ("C".into(), 30)].into()),
			]
			.into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 70);
		}
	}
}
