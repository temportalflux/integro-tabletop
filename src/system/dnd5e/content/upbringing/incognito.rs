use crate::{
	system::dnd5e::data::{
		mutator::{AddAbilityScore, AddProficiency, AddSkill},
		proficiency, Ability, Feature, Skill, Upbringing,
	},
	utility::Selector,
};
use enumset::EnumSet;

pub fn incognito() -> Upbringing {
	Upbringing {
		name: "Incognito".into(),
		description: "You were brought up by those who were not what they seemed.".into(),
		features: vec![
			Feature {
				name: "Ability Score Increase".into(),
				description: "Your Charisma score increases by 2. In addition, one ability score of your choice increases by 1.".into(),
				mutators: vec![
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
				mutators: vec![
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
				mutators: vec![
					AddProficiency::Language(Selector::Specific("Common".into())).into(),
					AddProficiency::Language(Selector::Any { id: Some("langA".into()) }).into(),
					AddProficiency::Language(Selector::Any { id: Some("langB".into()) }).into(),
				],
				..Default::default()
			}.into(),
		],
		..Default::default()
	}
}
