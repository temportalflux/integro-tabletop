use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IncMinSense(pub String, pub i32);

impl crate::utility::TraitEq for IncMinSense {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for IncMinSense {
	fn id() -> &'static str {
		"increase_min_sense"
	}
}

impl Mutator for IncMinSense {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.senses_mut().push_min(self.0.clone(), self.1, source);
	}
}

impl FromKDL<DnD5e> for IncMinSense {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let kind = node.get_str(value_idx.next())?.to_owned();
		let amount = node.get_i64(value_idx.next())? as i32;
		Ok(Self(kind, amount))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::dnd5e::{BoxedMutator, DnD5e};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			DnD5e::defaultmut_parse_kdl::<IncMinSense>(doc)
		}

		#[test]
		fn valid_format() -> anyhow::Result<()> {
			let doc = "mutator \"increase_min_sense\" \"Darkvision\" 60";
			let expected = IncMinSense("Darkvision".into(), 60);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			Feature,
		};

		fn character(mutators: Vec<(&'static str, IncMinSense)>) -> Character {
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
		fn set_max() {
			let character = character(vec![("TestFeature", IncMinSense("Darkvision".into(), 60))]);
			assert_eq!(
				character.senses().get("Darkvision"),
				Some(&(60, [("TestFeature".into(), 60)].into()).into())
			);
		}

		#[test]
		fn extend_max() {
			let character = character(vec![
				("SenseB", IncMinSense("Darkvision".into(), 60)),
				("SenseA", IncMinSense("Darkvision".into(), 40)),
			]);
			assert_eq!(
				character.senses().get("Darkvision"),
				Some(&(60, [("SenseA".into(), 40), ("SenseB".into(), 60)].into()).into())
			);
		}
	}
}

