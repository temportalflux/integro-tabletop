use crate::{
	system::dnd5e::{
		data::{
			condition::{self, Condition},
			evaluator::armor::HasArmorEquipped,
			item::{
				armor,
				weapon::{self},
			},
			mutator::{
				self, AddArmorClassFormula, AddDefense, AddProficiency, AddSavingThrowModifier,
			},
			proficiency,
			roll::Die,
			Ability, ArmorClassFormula, Class, DamageType, Feature, Level, Skill, Subclass,
			WeaponProficiency,
		},
		Value,
	},
	utility::Selector,
};

pub fn barbarian(levels: usize, subclass: Option<Subclass>) -> Class {
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
				/* TODO
				BonusDamage {
					amount: Value::Evaluated(
						ByLevel {
							class_name: Some("Barbarian".into()),
							map: [(1, "2".into()), (9, "3".into()), (16, "4".into())].into(),
						}
						.into(),
					),
					restriction: Some(weapon::Restriction {
						attack_kind: [AttackKind::Melee].into(),
						ability: [Ability::Strength].into(),
						..Default::default()
					}),
				}
				.into(),
				*/
				// TODO: AddAbilityModifier::Advantage(Ability::Strength).into(),
				AddSavingThrowModifier {
					ability: Some(Ability::Strength),
					target: None,
				}
				.into(),
				AddDefense {
					defense: mutator::Defense::Resistance,
					damage_type: Some(Value::Fixed(DamageType::Bludgeoning)),
					..Default::default()
				}
				.into(),
				AddDefense {
					defense: mutator::Defense::Resistance,
					damage_type: Some(Value::Fixed(DamageType::Piercing)),
					..Default::default()
				}
				.into(),
				AddDefense {
					defense: mutator::Defense::Resistance,
					damage_type: Some(Value::Fixed(DamageType::Slashing)),
					..Default::default()
				}
				.into(),
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
				let mut desc = "In battle, you fight with primal ferocity. On your turn, you can enter a rage as a bonus action.\n".into();
				desc += condition.description().as_str();
				desc += "\nYour rage lasts for 1 minute. It ends early if you are knocked unconscious or if your turn ends and you haven't attacked \
				a hostile creature since your last turn or taken damage since then. You can also end your rage on your turn as a bonus action.

				Once you have used all your rages, you must finish a long rest before you can rage again.";
				desc
			},
			/* TODO
			limited_uses: Some(LimitedUses {
				max_uses: Value::Evaluated(
					ByLevel {
						class_name: Some("Barbarian".into()),
						map: [(1, "2"), (3, "3"), (6, "4"), (12, "5"), (17, "6"), (20, "")].into(),
					}
					.into(),
				),
				reset_on: Some(Rest::Long),
				apply_conditions: vec![condition.into()],
			}),
			*/
			..Default::default()
		}
	};
	let mut class = Class {
		name: "Barbarian".into(),
		hit_die: Die::D12,
		subclass_selection_level: 3,
		subclass,
		levels: vec![],
	};

	if levels >= 1 {
		class.levels.push(Level {
			mutators: vec![
				AddProficiency::Armor(armor::Kind::Light).into(),
				AddProficiency::Armor(armor::Kind::Medium).into(),
				AddProficiency::Armor(armor::Kind::Heavy).into(),
				AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple)).into(),
				AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial)).into(),
				AddProficiency::SavingThrow(Ability::Strength).into(),
				AddProficiency::SavingThrow(Ability::Constitution).into(),
				AddProficiency::Skill(
					Selector::AnyOf {
						id: Some("skillA").into(),
						options: vec![
							Skill::AnimalHandling,
							Skill::Athletics,
							Skill::Intimidation,
							Skill::Nature,
							Skill::Perception,
							Skill::Survival,
						],
					},
					proficiency::Level::Full
				).into(),
				AddProficiency::Skill(
					Selector::AnyOf {
						id: Some("skillB").into(),
						options: vec![
							Skill::AnimalHandling,
							Skill::Athletics,
							Skill::Intimidation,
							Skill::Nature,
							Skill::Perception,
							Skill::Survival,
						],
					},
					proficiency::Level::Full
				).into(),
			],
			features: vec![
				rage.into(),
				Feature {
					name: "Unarmored Defense".into(),
					description: "While you are not wearing any armor, your Armor Class equals 10 + your Dexterity modifier + your Constitution modifier. You can use a shield and still gain this benefit.".into(),
					mutators: vec![
						AddArmorClassFormula(ArmorClassFormula {
							base: 10,
							bonuses: vec![Ability::Dexterity.into(), Ability::Constitution.into()],
						}).into()
					],
					criteria: Some(HasArmorEquipped {
						inverted: true,
						..Default::default()
					}.into()),
					..Default::default()
				}.into(),
			],
			..Default::default()
		});
	}
	if levels >= 2 {
		class.levels.push(Level {
			features: vec![
				Feature {
					name: "Reckless Attack".into(),
					description:
						"Starting at 2nd level, you can throw aside all concern for defense \
					to attack with fierce desperation. When you make your first attack on your turn, \
					you can decide to attack recklessly. Doing so gives you advantage on melee weapon \
					attack rolls using Strength during this turn, but attack rolls against you have \
					advantage until your next turn."
							.into(),
					..Default::default()
				}
				.into(),
				Feature {
					name: "Danger Sense".into(),
					description:
						"At 2nd level, you gain an uncanny sense of when things nearby aren't \
						as they should be, giving you an edge when you dodge away from danger.
						You have advantage on Dexterity saving throws against effects that you can see, \
						such as traps and spells. To gain this benefit, you can't be blinded, deafened, or incapacitated."
							.into(),
					mutators: vec![AddSavingThrowModifier {
						ability: Some(Ability::Dexterity),
						target: Some("effects you can see".into()),
					}
					.into()],
					/*
					criteria: Some(HasAnyCondition { inverted: true, values: vec![
						"Blinded".into(),
						"Deafened".into(),
						"Incapacitated".into(),
					] }),
					*/
					..Default::default()
				}
				.into(),
			],
			..Default::default()
		});
	}
	if levels >= 3 {
		class.levels.push(Level::default());
	}
	if levels >= 4 {
		// ASI
		class.levels.push(Level {
			..Default::default()
		});
	}
	if levels >= 5 {
		class.levels.push(Level {
			features: vec![
				Feature {
					name: "Extra Attack".into(),
					description: "Beginning at 5th level, you can attack twice, instead of once, \
					whenever you take the Attack action on your turn."
						.into(),
					mutators: vec![
					// TODO: AddAttacksPerAction(1).into(),
					],
					..Default::default()
				}
				.into(),
				Feature {
					name: "Fast Movement".into(),
					description: "Starting at 5th level, your speed increases by 10 feet \
				while you aren't wearing heavy armor."
						.into(),
					mutators: vec![
					// TODO: AddMovementSpeed(10).into(),
					],
					criteria: Some(
						HasArmorEquipped {
							inverted: true,
							kinds: [armor::Kind::Heavy].into(),
						}
						.into(),
					),
					..Default::default()
				}
				.into(),
			],
			..Default::default()
		});
	}
	class
}
