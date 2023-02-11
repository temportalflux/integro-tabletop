use super::StatsBuilder;
use crate::system::dnd5e::{
	condition::Condition,
	modifier::{self, Modifier},
	roll::Die,
	Action, Feature, LimitedUses,
};

#[derive(Clone, PartialEq)]
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

#[allow(dead_code)]
fn barbarian() {
	#[allow(dead_code)]
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

	#[allow(dead_code)]
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

#[allow(dead_code)]
fn monk() {

	/* Unarmored Defense
	Feature {
		description: "Beginning at 1st level, while you are wearing no armor and not wielding a shield, your AC equals 10 + your Dexterity modifier + your Wisdom modifier.".into(),
		modifiers: vec![ Box::new(AddArmorClass(Value::Evaluated(AbilityModifier(Ability::Wisdom)))) ],
		restriction: Some(!IsArmorEquipped + !IsShieldEqupped),
	}
	*/

	/* Martial Arts
	"At 1st level, your practice of martial arts gives you mastery of combat styles that use \
	unarmed strikes and monk weapons, which are shortswords and any simple melee weapons \
	that don’t have the two-handed or heavy property.

	You gain the following benefits while you are unarmed or wielding only monk weapons and \
	you aren’t wearing armor or wielding a shield:
	- You can use Dexterity instead of Strength for the attack and damage rolls of your unarmed strikes and monk weapons.
	- You can roll a d4 in place of the normal damage of your unarmed strike or monk weapon. \
	This die changes as you gain monk levels, as shown in the Martial Arts column of the Monk table.
	- When you use the Attack action with an unarmed strike or a monk weapon on your turn, you can make \
	one unarmed strike as a bonus action. For example, if you take the Attack action and attack with a quarterstaff, \
	you can also make an unarmed strike as a bonus action, assuming you haven’t already taken a bonus action this turn.

	Certain monasteries use specialized forms of the monk weapons. For example, you might use a club that is \
	two lengths of wood connected by a short chain (called a nunchaku) or a sickle with a shorter, \
	straighter blade (called a kama). Whatever name you use for a monk weapon, you can use the game statistics \
	provided for the weapon in the Weapons section."

	let is_monk_weapon = WeaponRestriction {
		(Specific(Shortsword) || Type(SimpleMelee)) && !IsTwoHanded && !IsHeavy
	}

	// For Unarmed attacks and monk weapons:
	// - use max modifier of STR || DEX
	// - use max die type of <attack dmg die> || d4
	// if:
	// - not armored
	// - no shield

	Modifiers:
	- GiveAlternativeDamageAbility { ability: Ability::Dexterity, restriction: Some(All(vec![HasProficiencyWithWeapon, Not(IsArmorEquipped), Not(IsShieldEquipped)])) }


	*/

	/* Ki
	Starting at 2nd level, your training allows you to harness the mystic energy of ki. Your access to this energy is \
	represented by a number of ki points. Your monk level determines the number of points you have, as shown in the \
	Ki Points column of the Monk table.
	You can spend these points to fuel various ki features. You start knowing three such features: \
	Flurry of Blows, Patient Defense, and Step of the Wind. You learn more ki features as you gain levels in this class.
	When you spend a ki point, it is unavailable until you finish a short or long rest, \
	at the end of which you draw all of your expended ki back into yourself. You must spend at least \
	30 minutes of the rest meditating to regain your ki points.
	Some of your ki features require your target to make a saving throw to resist the feature’s effects. \
	The saving throw DC is calculated as follows:
	Ki save DC = 8 + your proficiency bonus + your Wisdom modifier

	Feature {
		name: "Ki".into(),
		description: "".into(),
		uses: Some(LimitedUse::Resource {
			// enum Value<T> { Fixed(T), Evaluated(Evaluator<Item=T>) }
			max_uses: Some(Value::Evaluated(ClassLevel)),
			reset_on: Rest::Short,
		})
	}

	Feature {
		name: "Flurry of Blows".into(),
		description: "Immediately after you take the Attack action on your turn, you can spend 1 ki point to make two unarmed strikes as a bonus action.".into(),
		action: Some(Action::Bonus),
		uses: Some(LimitedUse::DependsOn {
			feature: "Monk/Ki",
			cost: 1,
		})
	}

	Feature {
		name: "Patient Defense".into(),
		description: "You can spend 1 ki point to take the Dodge action as a bonus action on your turn.".into(),
		action: Some(Action::Bonus),
		uses: Some(LimitedUse::DependsOn {
			feature: "Monk/Ki",
			cost: 1,
		})
	}

	Feature {
		name: "Step of the Wind".into(),
		description: "You can spend 1 ki point to take the Disengage or Dash action as a bonus action on your turn, and your jump distance is doubled for the turn.".into(),
		action: Some(Action::Bonus),
		uses: Some(LimitedUse::DependsOn {
			feature: "Monk/Ki",
			cost: 1,
		}),
	}

	Feature {
		name: "Deflect Missiles".into(),
		// TODO: Format "1d10 + your Dexterity modifier + your monk level" to "1d10 + n"
		description: "Starting at 3rd level, you can use your reaction to deflect or catch the missile when \
		you are hit by a ranged weapon attack. When you do so, the damage you take from the attack is \
		reduced by 1d10 + your Dexterity modifier + your monk level.
		If you reduce the damage to 0, you can catch the missile if it is small enough for you to hold in \
		one hand and you have at least one hand free. If you catch a missile in this way, you can \
		spend 1 ki point to make a ranged attack with the weapon or piece of ammunition you just caught, \
		as part of the same reaction. You make this attack with proficiency, regardless of your \
		weapon proficiencies, and the missile counts as a monk weapon for the attack, which has a \
		normal range of 20 feet and a long range of 60 feet.".into(),
		action: Some(Action::Reaction),
		uses: Some(LimitedUse::DependsOn {
			feature: "Monk/Ki",
			cost: 1,
		}),
		attack: Some(Attack {
			proficient,
			ability is max(str, dex)
			ranged of (20, 60)
		}),
	}

	Feature {
		name: "Stunning Strike".into(),
		// TODO: Embed con-save DC of (8 + your proficiency bonus + your Wisdom modifier)
		description: "Starting at 5th level, you can interfere with the flow of ki in an opponent's body. \
		When you hit another creature with a melee weapon attack, you can spend 1 ki point to attempt a stunning strike. \
		The target must succeed on a Constitution saving throw or be stunned until the end of your next turn.".into(),
		action: None,
		uses: Some(LimitedUse::DependsOn {
			feature: "Monk/Ki",
			cost: 1,
		}),
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
