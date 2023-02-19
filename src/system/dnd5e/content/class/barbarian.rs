use crate::system::dnd5e::{
	character::{AddProficiency, Class, Level, WeaponProficiency},
	condition::{self, Condition},
	criteria::armor::HasArmorEquipped,
	evaluator::ByClassLevel,
	item::{
		armor,
		weapon::{self, AttackType},
	},
	mutator::{self, AddDefense, AddSavingThrow, AddSkill, BonusDamage},
	proficiency,
	roll::Die,
	Ability, Action, Feature, LimitedUses, Rest, Skill, Value,
};

pub fn barbarian() -> Class {
	let rage = {
		let desc = "While raging, you gain the following benefits \
		if you aren't wearing heavy armor:
		- You have advantage on Strength checks and Strength saving throws.
		- When you make a melee weapon attack using Strength, you gain a bonus to the damage roll.
		- You have resistance to bludgeoning, piercing, and slashing damage.
		
		If you are able to cast spells, you can't cast them or concentrate on them while raging.";
		let condition = condition::Custom {
			name: "Raging".into(),
			description: desc.into(),
			mutators: vec![
				BonusDamage {
					amount: crate::system::dnd5e::Value::Evaluated(
						ByClassLevel::from([(1, 2), (9, 3), (16, 4)]).into(),
					),
					restriction: Some(weapon::Restriction {
						attack_kind: vec![AttackType::Melee],
						ability: vec![Ability::Strength],
						..Default::default()
					}),
				}
				.into(),
				// TODO: AddAbilityModifier::Advantage(Ability::Strength).into(),
				AddSavingThrow::Advantage(Ability::Strength, None).into(),
				AddDefense(mutator::Defense::Resistant, "Bludgeoning".into()).into(),
				AddDefense(mutator::Defense::Resistant, "Piercing".into()).into(),
				AddDefense(mutator::Defense::Resistant, "Slashing".into()).into(),
			],
			criteria: Some(
				HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Heavy].into(),
				}
				.into(),
			),
		};

		Feature {
			name: "Rage".into(),
			description: {
				let mut desc = "In battle, you fight with primal ferocity. On your turn, you can enter a rage as a bonus action.".into();
				desc += condition.description().as_str();
				desc += "Your rage lasts for 1 minute. It ends early if you are knocked unconscious or if your turn ends and you haven't attacked \
				a hostile creature since your last turn or taken damage since then. You can also end your rage on your turn as a bonus action.

				Once you have used all your rages, you must finish a long rest before you can rage again.";
				desc
			},
			action: Some(Action::Bonus),
			limited_uses: Some(LimitedUses {
				max_uses: Value::Evaluated(
					ByClassLevel::from([
						(1, Some(2)),
						(3, Some(3)),
						(6, Some(4)),
						(12, Some(5)),
						(17, Some(6)),
						(20, None),
					])
					.into(),
				),
				reset_on: Some(Rest::Long),
				apply_conditions: vec![condition.into()],
			}),
			..Default::default()
		}
	};
	let class = Class {
		name: "Barbarian".into(),
		hit_die: Die::D12,
		subclass_selection_level: 3,
		subclass: None,
		levels: vec![Level {
			mutators: vec![
				AddProficiency::Armor(armor::Kind::Light).into(),
				AddProficiency::Armor(armor::Kind::Medium).into(),
				AddProficiency::Armor(armor::Kind::Heavy).into(),
				AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple)).into(),
				AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial)).into(),
				AddSavingThrow::Proficiency(Ability::Strength).into(),
				AddSavingThrow::Proficiency(Ability::Constitution).into(),
				AddSkill {
					skill: mutator::Selector::AnyOf {
						id: Some("skillA".into()),
						options: vec![
							Skill::AnimalHandling,
							Skill::Athletics,
							Skill::Intimidation,
							Skill::Nature,
							Skill::Perception,
							Skill::Survival,
						],
					},
					proficiency: proficiency::Level::Full,
				}
				.into(),
				AddSkill {
					skill: mutator::Selector::AnyOf {
						id: Some("skillB".into()),
						options: vec![
							Skill::AnimalHandling,
							Skill::Athletics,
							Skill::Intimidation,
							Skill::Nature,
							Skill::Perception,
							Skill::Survival,
						],
					},
					proficiency: proficiency::Level::Full,
				}
				.into(),
			],
			features: vec![
				rage.into(),
				Feature {
					name: "Unarmored Defense".into(),
					description: "While you are not wearing any armor, your Armor Class equals 10 + your Dexterity modifier + your Constitution modifier. You can use a shield and still gain this benefit.".into(),
					mutators: vec![
						// TODO: AddArmorClassFormula { base: 10, modifiers: vec![Ability::Dexterity, Ability::Constitution] }.into()
					],
					criteria: Some(HasArmorEquipped {
						inverted: true,
						..Default::default()
					}.into()),
					..Default::default()
				}.into(),
			],
			..Default::default()
		},
		Level {
			features: vec![
				Feature {
					name: "Danger Sense".into(),
					description: "At 2nd level, you gain an uncanny sense of when things nearby aren't \
					as they should be, giving you an edge when you dodge away from danger.
					You have advantage on Dexterity saving throws against effects that you can see, \
					such as traps and spells. To gain this benefit, you can't be blinded, deafened, or incapacitated.".into(),
					mutators: vec![],
					/*
					criteria: Some(HasAnyCondition { inverted: true, values: vec![
						condition::Blinded,
						condition::Deafened,
						condition::Incapacitated,
					] }),
					*/
					..Default::default()
				}.into(),
			],
			..Default::default()
		}],
	};

	/* Extra Attack
	Feature {
		modifiers: vec![ Box::new(AddAttacksPerAction(1)) ]
	}
	*/

	/* Fast Movement
	Feature {
		modifiers: vec![ Box::new(AddMovementSpeed(10)) ],
		restriction: Some(IsOneArmorTypeEquippedOf(vec![
			None, Some(ArmorType::Light), Some(ArmorType::Medium),
		])),
	}
	*/

	class
}
