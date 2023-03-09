use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IncMinSpeed(pub String, pub i32);

impl crate::utility::TraitEq for IncMinSpeed {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for IncMinSpeed {
	fn id() -> &'static str {
		"increase_min_speed"
	}
}

impl Mutator for IncMinSpeed {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.speeds_mut().push_min(self.0.clone(), self.1, source);
	}
}

impl FromKDL<DnD5e> for IncMinSpeed {
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
			DnD5e::defaultmut_parse_kdl::<IncMinSpeed>(doc)
		}

		#[test]
		fn valid_format() -> anyhow::Result<()> {
			let doc = "mutator \"increase_min_speed\" \"Walking\" 30";
			let expected = IncMinSpeed("Walking".into(), 30);
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

		fn character(mutators: Vec<(&'static str, IncMinSpeed)>) -> Character {
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
			let character = character(vec![("TestFeature", IncMinSpeed("Walking".into(), 30))]);
			assert_eq!(
				character.speeds().get("Walking"),
				Some(&(30, [("TestFeature".into(), 30)].into()).into())
			);
		}

		#[test]
		fn extend_max() {
			let character = character(vec![
				("SpeedB", IncMinSpeed("Walking".into(), 35)),
				("SpeedA", IncMinSpeed("Walking".into(), 20)),
			]);
			assert_eq!(
				character.speeds().get("Walking"),
				Some(&(35, [("SpeedA".into(), 20), ("SpeedB".into(), 35)].into()).into())
			);
		}
	}
}
