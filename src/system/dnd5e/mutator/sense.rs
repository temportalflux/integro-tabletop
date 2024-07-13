use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::{
			data::{
				character::{Character, StatOperation},
				description,
			},
			mutator::StatMutator,
		},
		mutator::ReferencePath,
		Mutator,
	},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub struct Sense(pub StatMutator);

crate::impl_trait_eq!(Sense);
kdlize::impl_kdl_node!(Sense, "sense");

impl Mutator for Sense {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		let subject = match &self.0.stat_name {
			None => "All granted sense".to_owned(),
			Some(name) => format!("Your {name}"),
		};
		let content = format!("{subject} {}.", match &self.0.operation {
			StatOperation::MinimumValue(value) => format!("is at least {value} feet"),
			StatOperation::MinimumStat(value) => format!("is at least equivalent to your {value}"),
			StatOperation::Base(value) => format!("is at least {value} feet"),
			StatOperation::AddSubtract(value) if *value >= 0 => format!("increases by {value} feet"),
			StatOperation::AddSubtract(value) => format!("decreases by {value} feet"),
			StatOperation::MultiplyDivide(value) if *value >= 0 => format!("is multiplied by {value}"),
			StatOperation::MultiplyDivide(value) => format!("is dividied by {value}"),
			StatOperation::MaximumValue(value) => format!("is at most {value} feet"),
		});
		description::Section { content: content.into(), ..Default::default() }
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		stats.senses_mut().insert(self.0.stat_name.clone(), self.0.operation.clone(), parent);
	}
}

impl FromKdl<NodeContext> for Sense {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut crate::kdl_ext::NodeReader) -> anyhow::Result<Self> {
		Ok(Self(StatMutator::from_kdl(node)?))
	}
}

impl AsKdl for Sense {
	fn as_kdl(&self) -> NodeBuilder {
		self.0.as_kdl()
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
			let data =
				Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::MinimumValue(60) });
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn additive() -> anyhow::Result<()> {
			let doc = "mutator \"sense\" \"Darkvision\" (Add)60";
			let data = Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::AddSubtract(60) });
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			Bundle,
		};
		use std::path::PathBuf;

		fn character(mutators: Vec<(&'static str, Sense)>) -> Character {
			Character::from(Persistent {
				bundles: mutators
					.into_iter()
					.map(|(name, mutator)| {
						Bundle { name: name.into(), mutators: vec![mutator.into()], ..Default::default() }.into()
					})
					.collect(),
				..Default::default()
			})
		}

		#[test]
		fn minimum_single() {
			let character = character(vec![(
				"TestFeature",
				Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::MinimumValue(60) }),
			)]);
			let sense = character.senses().get("Darkvision").cloned().collect::<Vec<_>>();
			let expected: Vec<(_, PathBuf)> = vec![(StatOperation::MinimumValue(60), "TestFeature".into())];
			assert_eq!(sense, expected);
		}

		#[test]
		fn minimum_multiple() {
			let character = character(vec![
				(
					"SenseB",
					Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::MinimumValue(60) }),
				),
				(
					"SenseA",
					Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::MinimumValue(40) }),
				),
			]);
			let sense = character.senses().get("Darkvision").cloned().collect::<Vec<_>>();
			let expected: Vec<(_, PathBuf)> = vec![
				(StatOperation::MinimumValue(40), "SenseA".into()),
				(StatOperation::MinimumValue(60), "SenseB".into()),
			];
			assert_eq!(sense, expected);
		}

		#[test]
		fn single_additive() {
			let character = character(vec![(
				"TestFeature",
				Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::AddSubtract(60) }),
			)]);
			let sense = character.senses().get("Darkvision").cloned().collect::<Vec<_>>();
			let expected: Vec<(_, PathBuf)> = vec![(StatOperation::AddSubtract(60), "TestFeature".into())];
			assert_eq!(sense, expected);
		}

		#[test]
		fn minimum_gt_additive() {
			let character = character(vec![
				(
					"A",
					Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::MinimumValue(60) }),
				),
				("B", Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::AddSubtract(40) })),
			]);
			let sense = character.senses().get("Darkvision").cloned().collect::<Vec<_>>();
			let expected: Vec<(_, PathBuf)> =
				vec![(StatOperation::AddSubtract(40), "B".into()), (StatOperation::MinimumValue(60), "A".into())];
			assert_eq!(sense, expected);
		}

		#[test]
		fn minimum_lt_additive() {
			let character = character(vec![
				(
					"A",
					Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::MinimumValue(60) }),
				),
				("B", Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::AddSubtract(40) })),
				("C", Sense(StatMutator { stat_name: Some("Darkvision".into()), operation: StatOperation::AddSubtract(30) })),
			]);
			let sense = character.senses().get("Darkvision").cloned().collect::<Vec<_>>();
			let expected: Vec<(_, PathBuf)> = vec![
				(StatOperation::AddSubtract(40), "B".into()),
				(StatOperation::AddSubtract(30), "C".into()),
				(StatOperation::MinimumValue(60), "A".into()),
			];
			assert_eq!(sense, expected);
		}
	}
}
