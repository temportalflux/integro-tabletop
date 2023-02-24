use crate::system::dnd5e::{character::Character, Ability};

#[derive(Clone)]
pub enum AddSavingThrow {
	Proficiency(Ability),
	Advantage(Ability, Option<String>),
}

impl super::Mutator for AddSavingThrow {
	fn apply<'c>(&self, stats: &mut Character) {
		match self {
			Self::Proficiency(ability) => {
				let source = stats.source_path();
				stats.saving_throws_mut().add_proficiency(*ability, source);
			}
			Self::Advantage(ability, target) => {
				let source = stats.source_path();
				stats
					.saving_throws_mut()
					.add_modifier(*ability, target.clone(), source);
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::AddSavingThrow;
	use crate::system::dnd5e::{
		character::{Character, Persistent},
		proficiency, Ability, Feature,
	};

	#[test]
	fn proficiency() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddSavingThrow".into(),
				mutators: vec![AddSavingThrow::Proficiency(Ability::Wisdom).into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		let (prof, _) = &character.saving_throws()[Ability::Wisdom];
		assert_eq!(*prof.value(), proficiency::Level::Full);
		assert_eq!(
			*prof.sources(),
			vec![("AddSavingThrow".into(), proficiency::Level::Full)]
		);
	}

	#[test]
	fn advantage() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddSavingThrow".into(),
				mutators: vec![
					AddSavingThrow::Advantage(Ability::Wisdom, Some("Magic".into())).into(),
				],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		let (_, advantages) = &character.saving_throws()[Ability::Wisdom];
		assert_eq!(
			*advantages,
			vec![(Some("Magic".into()), "AddSavingThrow".into())]
		);
	}
}
