use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::{bounded::BoundValue, character::Character, description},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Speed {
	pub name: String,
	pub argument: BoundValue,
}

crate::impl_trait_eq!(Speed);
crate::impl_kdl_node!(Speed, "speed");

impl Mutator for Speed {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			content: format!(
				"Your {} speed {}.",
				self.name,
				match &self.argument {
					BoundValue::Minimum(value) => format!("is at least {value} feet"),
					BoundValue::Base(value) => format!("is at least {value} feet"),
					BoundValue::Additive(value) => format!("increases by {value} feet"),
					BoundValue::Subtract(value) => format!("decreases by {value} feet"),
				}
			),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		stats
			.speeds_mut()
			.insert(self.name.clone(), self.argument.clone(), parent.to_owned());
	}
}

impl FromKDL for Speed {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req(ctx.consume_idx())?.to_owned();
		let argument = BoundValue::from_kdl(node, ctx)?;
		Ok(Self { name, argument })
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::{core::NodeRegistry, dnd5e::BoxedMutator};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<Speed>(doc)
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
