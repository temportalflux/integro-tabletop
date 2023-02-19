use crate::system::dnd5e::{
	character::{ArmorClassFormula, BoundedAbility},
	item::{
		armor::{self, Armor},
		equipment::Equipment,
		Item, ItemKind,
	},
	mutator::AddSkillModifier,
	roll, Ability, Skill,
};

pub fn leather() -> Item {
	Item {
		name: "Leather Armor".into(),
		description: None,
		weight: 10,
		worth: 1000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			armor: Some(Armor {
				kind: armor::Kind::Light,
				formula: ArmorClassFormula {
					base: 11,
					bonuses: vec![Ability::Dexterity.into()],
				},
				min_strength_score: None,
			}),
			..Default::default()
		}),
	}
}

pub fn scale_mail() -> Item {
	Item {
		name: "Scale Mail".into(),
		description: None,
		weight: 45,
		worth: 5000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![AddSkillModifier {
				skill: Skill::Stealth,
				modifier: roll::Modifier::Disadvantage,
				criteria: None,
			}
			.into()],
			armor: Some(Armor {
				kind: armor::Kind::Medium,
				formula: ArmorClassFormula {
					base: 14,
					bonuses: vec![BoundedAbility {
						ability: Ability::Dexterity,
						max: Some(2),
						min: None,
					}],
				},
				min_strength_score: None,
			}),
			..Default::default()
		}),
	}
}

pub fn splint() -> Item {
	Item {
		name: "Splint".into(),
		description: None,
		weight: 60,
		worth: 20000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![AddSkillModifier {
				skill: Skill::Stealth,
				modifier: roll::Modifier::Disadvantage,
				criteria: None,
			}
			.into()],
			armor: Some(Armor {
				kind: armor::Kind::Heavy,
				formula: ArmorClassFormula {
					base: 17,
					bonuses: vec![],
				},
				min_strength_score: Some(15),
			}),
			..Default::default()
		}),
	}
}
