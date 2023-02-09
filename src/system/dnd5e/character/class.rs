use super::StatsBuilder;
use crate::system::dnd5e::{modifier::{self, Modifier}, roll::Die, Action, Feature, condition::{Condition}, LimitedUses};

#[derive(Clone)]
pub struct Class {
	pub name: String,
	pub hit_die: Die,
}

impl Class {
	pub fn level_count(&self) -> i32 {
		// TODO
		0
	}
}

impl modifier::Container for Class {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}

	fn apply_modifiers<'c>(&self, _stats: &mut StatsBuilder<'c>) {}
}

fn barbarian() {
	let make_raging_condition = |damage_amt: i32| {
		CustomCondition {
			description: format!("While raging, you gain the following benefits if you aren't wearing heavy armor:
			You have advantage on Strength checks and Strength saving throws.
			When you make a melee weapon attack using Strength, you gain a +{damage_amt} bonus to the damage roll.
			You have resistance to bludgeoning, piercing, and slashing damage.
			
			If you are able to cast spells, you can't cast them or concentrate on them while raging."),
			modifiers: vec![
				Box::new(BonusDamage {
					amount: damage_amt,
					restriction: Some(() /*TODO: weapon is melee using strength*/),
				}),
				// TODO: adv on strength checks and saving throws
				// TODO: restistance to bludgeoning, piercing, and slashing damage
			],
			restriction: Some(() /*TODO: Not wearing heavy armor*/),
		}
	};
	
	let make_rage_feature = |max_uses: Option<u32>, condition: CustomCondition| {
		Feature {
			name: "Rage".into(),
			description: {
				let mut desc = "In battle, you fight with primal ferocity. On your turn, you can enter a rage as a bonus action.".into();
				desc += condition.description().as_str();
				desc += "Your rage lasts for 1 minute. It ends early if you are knocked unconscious or if your turn ends and you haven't attacked \
				a hostile creature since your last turn or taken damage since then. You can also end your rage on your turn as a bonus action.";
				if let Some(uses) = &max_uses {
					desc += format!("Once you have raged {uses} times, you must finish a long rest before you can rage again.").as_str();
				}
				desc
			},
			action: Some(Action::Bonus),
			modifiers: vec![],
			limited_uses: max_uses.map(|max_uses| LimitedUses {
				// TODO: max_uses
				// TODO: Resets on long rest
				// TODO: Conditions applied on use, which should have a "stop" button to clear the effects
			}),
		}
	};

	/*
	rage => uses: Uses {
		// enum Value<T> { Fixed(T), Evaluated(Evaluator<Item=T>) }
		max_uses: Some(Value::Evaluated(ByClassLevel(hash_map! {
			1 => Some(2),
			3 => Some(3),
			6 => Some(4),
			12 => Some(5),
			17 => Some(6),
			20 => None,
		}))),
		reset_on: Rest::Long,
		apply_conditions: vec![
			Box::new(CustomCondition {
				modifiers: vec![
					Box::new(BonusWeaponAttackDamage {
						// enum Value<T> { Fixed(T), Evaluated(Evaluator<Item=T>) }
						amount: Value::Evaluated(ByClassLevel(hash_map! {
							1 => 2,
							9 => 3,
							16 => 4,
						})),
						restriction: Some(WeaponRestriction {
							target: Some(WeaponType::Melee),
							ability: Some(Ability::Strength),
						}),
					}),
					Box::new(GiveAbilityAdvantage(Ability::Strength)),
					Box::new(GiveSavingThrowAdvantage(Ability::Strength, None)),
					Box::new(GiveDefense {
						defense: Defense::Resistance,
						damages: vec![
							DamageType::Blugeoning, DamageType::Piercing, DamageType::Slashing
						],
					}),
				],
				// do not apply modifiers if wearing heavy armor
				restriction: Some(IsOneArmorTypeEquippedOf(vec![
					None, Some(ArmorType::Light), Some(ArmorType::Medium),
				])),
			})
		]
	}
	*/

	/* Unarmored Defense
	"While you are not wearing any armor, your Armor Class equals 10 + your Dexterity modifier + your Constitution modifier. You can use a shield and still gain this benefit."
	NOTE: AC is 10 + dex by default, so this is just adding CON as a bonus to AC
	Feature {
		modifiers: vec![ Box::new(AddArmorClass(Value::Evaluated(AbilityModifier(Ability::Constitution)))) ],
		restriction: Some(IsOneArmorTypeEquippedOf(vec![None])),
	}
	*/

	/* Danger Sense
	"At 2nd level, you gain an uncanny sense of when things nearby aren’t as they should be, giving you an edge when you dodge away from danger.
	You have advantage on Dexterity saving throws against effects that you can see, such as traps and spells. To gain this benefit, you can’t be blinded, deafened, or incapacitated."
	Feature {
		modifiers: vec![
			Box::new(GiveSavingThrowAdvantage(Ability::Dexterity, Some("effects you can see (e.g. traps and spells)"))),
		],
		restriction: Some(AreAllConditionsInactive(vec![
			Condition::Blinded, Condition::Deafened, Condition::Incapacitated,
		]))
	}
	*/

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

	/* Primal Path (choose subclass)
	Feature {
		modifiers: vec![Box::new(
			SelectSubclass {
				// this is a modifier which has a special selector and saved value
				// it automatically generates options for each subclass of the current class as defined by the user's modules
				// the modifier will recurse through the subclass when evaluated
			}
		)],
	}
	*/
}

#[derive(Clone)]
struct CustomCondition {
	pub description: String,
	pub modifiers: Vec<Box<dyn Modifier + 'static>>,
	pub restriction: Option<()>, // TODO: check this to conditionally apply the modifiers
}
impl Condition for CustomCondition {
	fn description(&self) -> String {
		self.description.clone()
	}
}

#[derive(Clone, PartialEq)]
struct BonusDamage {
	amount: i32,
	restriction: Option<()>, // TODO: Evaluate for each weapon/attack before applying amount
}
impl Modifier for BonusDamage {
	fn scope_id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, stats: &mut StatsBuilder<'c>) {}
}
