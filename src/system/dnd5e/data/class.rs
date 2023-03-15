use super::{character::Character, mutator::AddMaxHitPoints, roll::Die, BoxedFeature};
use crate::{
	system::dnd5e::{BoxedMutator, Value},
	utility::{MutatorGroup, Selector},
};
use std::path::Path;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Class {
	pub name: String,
	pub hit_die: Die,
	pub levels: Vec<Level>,
	pub subclass_selection_level: usize,
	pub subclass: Option<Subclass>,
}

impl Class {
	pub fn level_count(&self) -> usize {
		self.levels.len()
	}

	fn iter_levels<'a>(&'a self) -> impl Iterator<Item = LevelWithIndex<'a>> + 'a {
		self.levels
			.iter()
			.enumerate()
			.map(|(idx, lvl)| LevelWithIndex(idx, lvl))
	}
}

impl MutatorGroup for Class {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_self = parent.join(&self.name);
		for level in self.iter_levels() {
			level.set_data_path(&path_to_self);
		}
		if let Some(subclass) = &self.subclass {
			subclass.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for level in self.iter_levels() {
			stats.apply_from(&level, &path_to_self);
		}
		if let Some(subclass) = &self.subclass {
			stats.apply_from(subclass, &path_to_self);
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Level {
	pub hit_points: Selector<u32>,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<BoxedFeature>,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			hit_points: Selector::Any {
				id: Some("hit_points").into(),
			},
			mutators: Default::default(),
			features: Default::default(),
		}
	}
}

struct LevelWithIndex<'a>(usize, &'a Level);
impl<'a> LevelWithIndex<'a> {
	fn level_name(&self) -> String {
		format!("level{:02}", self.0 + 1)
	}
}
impl<'a> MutatorGroup for LevelWithIndex<'a> {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(self.level_name());
		self.1.hit_points.set_data_path(&path_to_self);
		for mutator in &self.1.mutators {
			mutator.set_data_path(&path_to_self);
		}
		for feature in &self.1.features {
			feature.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(self.level_name());
		if let Some(hit_points) = stats.resolve_selector(&self.1.hit_points) {
			let mutator = AddMaxHitPoints {
				id: Some(format!("Level {:02}", self.0 + 1)),
				value: Value::Fixed(hit_points as i32),
			};
			stats.apply(&mutator.into(), &path_to_self);
		}
		for mutator in &self.1.mutators {
			stats.apply(mutator, &path_to_self);
		}
		for feature in &self.1.features {
			stats.add_feature(feature, &path_to_self);
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Subclass {
	pub name: String,
	pub levels: Vec<Level>,
}
impl Subclass {
	fn iter_levels<'a>(&'a self) -> impl Iterator<Item = LevelWithIndex<'a>> + 'a {
		self.levels
			.iter()
			.enumerate()
			.map(|(idx, lvl)| LevelWithIndex(idx, lvl))
	}
}
impl MutatorGroup for Subclass {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for level in self.iter_levels() {
			level.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for level in self.iter_levels() {
			stats.apply_from(&level, &path_to_self);
		}
	}
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
