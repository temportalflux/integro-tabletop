use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{
		data::{bounded::BoundValue, character::Character},
		DnD5e, FromKDL, KDLNode,
	},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Speed {
	pub name: String,
	pub argument: BoundValue,
}

impl crate::utility::TraitEq for Speed {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for Speed {
	fn id() -> &'static str {
		"speed"
	}
}

impl Mutator for Speed {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats
			.speeds_mut()
			.insert(self.name.clone(), self.argument.clone(), source);
	}
}

impl FromKDL<DnD5e> for Speed {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let name = node.get_str(value_idx.next())?.to_owned();
		let argument = BoundValue::from_kdl(node, value_idx, system)?;
		Ok(Self { name, argument })
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::dnd5e::{BoxedMutator, DnD5e};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			DnD5e::defaultmut_parse_kdl::<Speed>(doc)
		}

		#[test]
		fn minimum() -> anyhow::Result<()> {
			let doc = "mutator \"speed\" \"Walking\" (Minimum)30";
			let expected = Speed {
				name: "Walking".into(),
				argument: BoundValue::Minimum(30),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn additive() -> anyhow::Result<()> {
			let doc = "mutator \"speed\" \"Walking\" (Additive)30";
			let expected = Speed {
				name: "Walking".into(),
				argument: BoundValue::Additive(30),
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

		fn character(mutators: Vec<(&'static str, Speed)>) -> Character {
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
				Speed {
					name: "Walking".into(),
					argument: BoundValue::Minimum(60),
				},
			)]);
			let sense = character.speeds().get("Walking").unwrap();
			let expected = [(BoundKind::Minimum, [("TestFeature".into(), 60)].into())].into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 60);
		}

		#[test]
		fn minimum_multiple() {
			let character = character(vec![
				(
					"B",
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Minimum(60),
					},
				),
				(
					"A",
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Minimum(40),
					},
				),
			]);
			let sense = character.speeds().get("Walking").unwrap();
			let expected = [(
				BoundKind::Minimum,
				[("A".into(), 40), ("B".into(), 60)].into(),
			)]
			.into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 60);
		}

		#[test]
		fn single_additive() {
			let character = character(vec![(
				"TestFeature",
				Speed {
					name: "Walking".into(),
					argument: BoundValue::Additive(20),
				},
			)]);
			let sense = character.speeds().get("Walking").unwrap();
			let expected = [(BoundKind::Additive, [("TestFeature".into(), 20)].into())].into();
			assert_eq!(sense, &expected);
			assert_eq!(sense.value(), 20);
		}

		#[test]
		fn minimum_gt_additive() {
			let character = character(vec![
				(
					"A",
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Minimum(60),
					},
				),
				(
					"B",
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Additive(40),
					},
				),
			]);
			let sense = character.speeds().get("Walking").unwrap();
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
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Minimum(60),
					},
				),
				(
					"B",
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Additive(40),
					},
				),
				(
					"C",
					Speed {
						name: "Walking".into(),
						argument: BoundValue::Additive(30),
					},
				),
			]);
			let sense = character.speeds().get("Walking").unwrap();
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
