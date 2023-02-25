use crate::system::dnd5e::data::{
	action::ActivationKind,
	mutator::{AddLifeExpectancy, AddMaxHeight, AddMaxSpeed},
	roll::{Die, Roll},
	Feature, Lineage,
};

pub fn shapechanger() -> Lineage {
	Lineage {
		name: "Changeling (Shapechanger)".to_owned(),
		description: "One of your birth parents is a changeling. You can change your appearance at will.
		- Your life expectancy increases by 50 years.
		- Your height increases by 30 + 1d4 inches.".to_owned(),
		mutators: vec![
			AddLifeExpectancy(50).into(),
			AddMaxHeight::Value(30).into(),
			AddMaxHeight::Roll(Roll {
				amount: 1,
				die: Die::D4,
			})
			.into(),
			AddMaxSpeed("Walking".into(), 30).into(),
		],
		features: vec![
			Feature {
				name: "Shapechanger".to_owned(),
				description: "As an action, you can change your appearance. You determine the specifics of the changes, \
				including your coloration, hair length, and sex. You can also adjust your height and weight, \
				but not so much that your size changes. While shapechanged, none of your game statistics change. \
				You can't duplicate the appearance of a creature you've never seen, and you must adopt a form that has \
				the same basic arrangement of limbs that you have. Your voice, clothing, and equipment aren't changed by this trait. \
				You stay in the new form until you use an action to revert to your true form or until you die.".into(),
				action: Some(ActivationKind::Action),
				..Default::default()
			}.into(),
		],
		..Default::default()
	}
}

pub fn voice_changer() -> Lineage {
	Lineage {
		name: "Changeling (Voice Changer)".to_owned(),
		description: "One of your birth parents is a changeling. You can perfectly mimic another person's voice.".to_owned(),
		mutators: vec![
			AddLifeExpectancy(50).into(),
			AddMaxHeight::Value(30).into(),
			AddMaxHeight::Roll(Roll {
				amount: 1,
				die: Die::D4,
			})
			.into(),
			AddMaxSpeed("Walking".into(), 30).into(),
		],
		features: vec![
			Feature {
				name: "Voice Change".to_owned(),
				description: "As an action, you can change your voice. You can't duplicate the voice of a creature you've never heard. \
				Your appearance remains the same. You keep your mimicked voice until you use an action to revert to your true voice.".into(),
				action: Some(ActivationKind::Action),
				..Default::default()
			}.into(),
		],
		..Default::default()
	}
}
