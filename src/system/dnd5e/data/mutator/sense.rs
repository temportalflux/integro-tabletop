use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{bounded::BoundValue, character::Character},
			FromKDL, KDLNode,
		},
	},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Sense {
	pub name: String,
	pub argument: BoundValue,
}

impl crate::utility::TraitEq for Sense {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for Sense {
	fn id() -> &'static str {
		"sense"
	}
}

impl Mutator for Sense {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats
			.senses_mut()
			.insert(self.name.clone(), self.argument.clone(), source);
	}
}

impl FromKDL for Sense {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let name = node.get_str(value_idx.next())?.to_owned();
		let argument = BoundValue::from_kdl(node, value_idx, node_reg)?;
		Ok(Self { name, argument })
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::dnd5e::BoxedMutator;

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<Sense>(doc)
		}

		#[test]
		fn minimum() -> anyhow::Result<()> {
			let doc = "mutator \"sense\" \"Darkvision\" (Minimum)60";
			let expected = Sense {
				name: "Darkvision".into(),
				argument: BoundValue::Minimum(60),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn additive() -> anyhow::Result<()> {
			let doc = "mutator \"sense\" \"Darkvision\" (Additive)60";
			let expected = Sense {
				name: "Darkvision".into(),
				argument: BoundValue::Additive(60),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			bounded::BoundKind,
			character::{Character, Persistent},
			Feature,
		};

		fn character(mutators: Vec<(&'static str, Sense)>) -> Character {
			Character::from(Persistent {
				feats: mutators
					.into_iter()
					.map(|(name, mutator)| {
						Feature {
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
				(
					BoundKind::Additive,
					[("B".into(), 40), ("C".into(), 30)].into(),
				),
			]
			.into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 70);
		}
	}
}
