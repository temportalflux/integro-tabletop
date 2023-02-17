mod ability;
pub use ability::*;

mod action;
pub use action::*;

pub mod character;
pub mod condition;
pub mod criteria;

mod feature;
pub use feature::*;

pub mod modifier;

pub mod proficiency;

pub mod roll;

mod skill;
pub use skill::*;

pub mod hardcoded {
	use super::{modifier::AddMaxSpeed, *};

	fn changeling_age() -> Feature {
		use modifier::AddLifeExpectancy;
		Feature {
			name: "Age".into(),
			description: "Your life expectancy increases by 50 years.".into(),
			modifiers: vec![AddLifeExpectancy(50).into()],
			..Default::default()
		}
	}

	fn changeling_size() -> Feature {
		use modifier::AddMaxHeight;
		use roll::{Die, Roll};

		Feature {
			name: "Size".into(),
			description: "Your height increases by 30 + 1d4 inches.".into(),
			modifiers: vec![
				AddMaxHeight::Value(30).into(),
				AddMaxHeight::Roll(Roll {
					amount: 1,
					die: Die::D4,
				})
				.into(),
			],
			..Default::default()
		}
	}

	pub fn changeling1() -> character::Lineage {
		character::Lineage {
			name: "Changeling I".to_owned(),
			description: "One of your birth parents is a changeling. You can change your appearance at will.".to_owned(),
			features: vec![
				changeling_age().into(),
				changeling_size().into(),
				Feature {
					name: "Speeds".into(),
					modifiers: vec![
						AddMaxSpeed("Walking".into(), 30).into(),
					],
					..Default::default()
				}.into(),
				Feature {
					name: "Shapechanger".to_owned(),
					description: "As an action, you can change your appearance. You determine the specifics of the changes, \
					including your coloration, hair length, and sex. You can also adjust your height and weight, \
					but not so much that your size changes. While shapechanged, none of your game statistics change. \
					You can't duplicate the appearance of a creature you've never seen, and you must adopt a form that has \
					the same basic arrangement of limbs that you have. Your voice, clothing, and equipment aren't changed by this trait. \
					You stay in the new form until you use an action to revert to your true form or until you die.".into(),
					action: Some(Action::Full),
					..Default::default()
				}.into(),
			],
		}
	}

	pub fn changeling2() -> character::Lineage {
		character::Lineage {
			name: "Changeling II".to_owned(),
			description: "One of your birth parents is a changeling. You can perfectly mimic another person's voice.".to_owned(),
			features: vec![
				changeling_age().into(),
				changeling_size().into(),
				Feature {
					name: "Voice Change".to_owned(),
					description: "As an action, you can change your voice. You can't duplicate the voice of a creature you've never heard. \
					Your appearance remains the same. You keep your mimicked voice until you use an action to revert to your true voice.".into(),
					action: Some(Action::Full),
					..Default::default()
				}.into(),
			],
		}
	}

	pub fn incognito() -> character::Upbringing {
		use enumset::EnumSet;
		use modifier::{AddAbilityScore, AddLanguage, AddSkill, Selector};
		character::Upbringing {
			name: "Incognito".into(),
			description: "You were brought up by those who were not what they seemed.".into(),
			features: vec![
				Feature {
					name: "Ability Score Increase".into(),
					description: "Your Charisma score increases by 2. In addition, one ability score of your choice increases by 1.".into(),
					modifiers: vec![
						AddAbilityScore {
							ability: Selector::Specific(Ability::Charisma),
							value: 2,
						}.into(),
						AddAbilityScore {
							ability: Selector::AnyOf {
								id: None,
								options: EnumSet::only(Ability::Charisma).complement().into_iter().collect(),
							},
							value: 1,
						}.into(),
					],
					..Default::default()
				}.into(),
				Feature {
					name: "Good with People".into(),
					description: "You gain proficiency with two of the following skills of your choice: Deception, Insight, Intimidation, and Persuasion.".into(),
					modifiers: vec![
						AddSkill {
							skill: Selector::AnyOf {
								id: None,
								options: vec![
									Skill::Deception, Skill::Insight, Skill::Intimidation, Skill::Persuasion,
								],
							},
							proficiency: proficiency::Level::Full,
						}.into(),
					],
					..Default::default()
				}.into(),
				Feature {
					name: "Languages".into(),
					description: "You can speak, read, and write Common and two other languages of your choice.".into(),
					modifiers: vec![
						AddLanguage(Selector::Specific("Common".into())).into(),
						AddLanguage(Selector::Any { id: Some("langA".into()) }).into(),
						AddLanguage(Selector::Any { id: Some("langB".into()) }).into(),
					],
					..Default::default()
				}.into(),
			],
		}
	}

	pub fn anthropologist() -> character::Background {
		use modifier::{AddLanguage, AddSkill, Selector};
		character::Background {
			name: "Anthropologist".into(),
			description: "You have always been fascinated by other cultures, from the most ancient and \
			primeval lost lands to the most modern civilizations. By studying other cultures' customs, philosophies, \
			laws, rituals, religious beliefs, languages, and art, you have learned how tribes, empires, \
			and all forms of society in between craft their own destinies and doom. This knowledge came to \
			you not only through books and scrolls, but also through first-hand observation—by visiting far-flung \
			settlements and exploring local histories and customs.".into(),
			features: vec![
				Feature {
					name: "Skill Proficiencies".into(),
					modifiers: vec![
						AddSkill {
							skill: Selector::Specific(Skill::Insight),
							proficiency: proficiency::Level::Full,
						}.into(),
						AddSkill {
							skill: Selector::Specific(Skill::Religion),
							proficiency: proficiency::Level::Full,
						}.into(),
					],
					..Default::default()
				}.into(),
				Feature {
					name: "Languages".into(),
					modifiers: vec![
						AddLanguage(Selector::Any { id: Some("langA".into()) }).into(),
						AddLanguage(Selector::Any { id: Some("langB".into()) }).into(),
					],
					..Default::default()
				}.into(),
				Feature {
					name: "Adept Linguist".into(),
					description: "You can communicate with humanoids who don't speak any language you know. \
					You must observe the humanoids interacting with one another for at least 1 day, \
					after which you learn a handful of important words, expressions, and gestures—enough \
					to communicate on a rudimentary level.".into(),
					..Default::default()
				}.into(),
			],
		}
	}
}
