use crate::{
	system::dnd5e::{data::character::Character, KDLNode},
	utility::Mutator,
};
use enum_map::Enum;

#[derive(Clone, Copy, PartialEq, Enum, Debug)]
pub enum Defense {
	Resistant,
	Immune,
	Vulnerable,
}

#[derive(Clone, Debug)]
pub struct AddDefense(pub Defense, pub String);
impl KDLNode for AddDefense {
	fn id() -> &'static str {
		"add_defense"
	}
}
impl Mutator for AddDefense {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.defenses_mut().push(self.0, self.1.clone(), source);
	}
}

#[cfg(test)]
mod test {
	use super::{AddDefense, Defense};
	use crate::system::dnd5e::data::{
		character::{Character, Persistent},
		Feature,
	};

	#[test]
	fn resistant() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense(Defense::Resistant, "Fire".into()).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Resistant],
			[("Fire".into(), ["AddDefense".into()].into())].into()
		);
	}

	#[test]
	fn immune() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense(Defense::Immune, "Cold".into()).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Immune],
			[("Cold".into(), ["AddDefense".into()].into())].into()
		);
	}

	#[test]
	fn vulnerable() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddDefense".into(),
				mutators: vec![AddDefense(Defense::Vulnerable, "Psychic".into()).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.defenses()[Defense::Vulnerable],
			[("Psychic".into(), ["AddDefense".into()].into())].into()
		);
	}
}
