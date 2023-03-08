use super::condition::BoxedCondition;
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::dnd5e::{
		data::{character::Character, roll::Roll, Ability},
		DnD5e, FromKDL, Value,
	},
	utility::Evaluator,
	GeneralError,
};
use std::{path::PathBuf, str::FromStr};
use uuid::Uuid;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Action {
	pub name: String,
	pub description: String,
	pub activation_kind: ActivationKind,
	pub attack: Option<Attack>,
	/// Dictates how many times this action can be used until it is reset.
	pub limited_uses: Option<LimitedUses>,
	/// Conditions applied when the action is used.
	pub apply_conditions: Vec<BoxedCondition>,
	// generated
	pub source: Option<ActionSource>,
}
impl FromKDL<DnD5e> for Action {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.map(str::to_owned)
			.unwrap_or_default();
		let activation_kind = ActivationKind::from_kdl(
			node.query_req("activation")?,
			&mut ValueIdx::default(),
			system,
		)?;
		let attack = match node.query("attack")? {
			None => None,
			Some(node) => Some(Attack::from_kdl(node, &mut ValueIdx::default(), system)?),
		};
		let limited_uses = match node.query("limited_uses")? {
			None => None,
			Some(node) => Some(LimitedUses::from_kdl(
				node,
				&mut ValueIdx::default(),
				system,
			)?),
		};
		// TODO: conditions applied on use
		let apply_conditions = Vec::new();
		Ok(Self {
			name,
			description,
			activation_kind,
			attack,
			limited_uses,
			apply_conditions,
			source: None,
		})
	}
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub enum ActivationKind {
	#[default]
	Action,
	Bonus,
	Reaction,
	Minute(u32),
	Hour(u32),
}
impl FromKDL<DnD5e> for ActivationKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			"Minute" => Ok(Self::Minute(node.get_i64(value_idx.next())? as u32)),
			"Hour" => Ok(Self::Hour(node.get_i64(value_idx.next())? as u32)),
			name => Err(GeneralError(format!(
				"Invalid action activation type {name:?}, expected \
				Action, Bonus, Reaction, Minute, or Hour."
			))
			.into()),
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActionSource {
	Item(Uuid),
	Feature(PathBuf),
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct LimitedUses {
	/// The number of uses the feature has until it resets.
	/// TODO: Use a ScalingUses instead of Value, which always scale in relation to some evaluator (in most cases, get_level)
	pub max_uses: Value<Option<usize>>,
	/// Consumed uses resets when the user takes at least this rest
	/// (a reset on a short rest will also reset on long rest).
	pub reset_on: Option<Rest>,
}
impl FromKDL<DnD5e> for LimitedUses {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let max_uses = {
			// Temporary code, until I can implement scaling uses
			let node = node.query_req("max_uses")?;
			let max_uses = node.get_i64(0)? as usize;
			Value::Fixed(Some(max_uses))
		};
		let reset_on = match node.query_str_opt("reset_on", 0)? {
			None => None,
			Some(str) => Some(Rest::from_str(str)?),
		};
		Ok(Self { max_uses, reset_on })
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Rest {
	Short,
	Long,
}
impl ToString for Rest {
	fn to_string(&self) -> String {
		match self {
			Self::Short => "Short",
			Self::Long => "Long",
		}
		.to_owned()
	}
}
impl FromStr for Rest {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Short" => Ok(Self::Short),
			"Long" => Ok(Self::Long),
			_ => Err(GeneralError(format!(
				"Invalid rest {s:?}, expected Short or Long"
			))),
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attack {
	pub kind: AttackKindValue,
	pub check: AttackCheckKind,
	pub area_of_effect: Option<AreaOfEffect>,
	pub damage: Option<DamageRoll>,
}

impl FromKDL<DnD5e> for Attack {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let kind =
			AttackKindValue::from_kdl(node.query_req("kind")?, &mut ValueIdx::default(), system)?;
		let check =
			AttackCheckKind::from_kdl(node.query_req("check")?, &mut ValueIdx::default(), system)?;
		let area_of_effect = match node.query("area_of_effect")? {
			None => None,
			Some(node) => Some(AreaOfEffect::from_kdl(
				node,
				&mut ValueIdx::default(),
				system,
			)?),
		};
		let damage = match node.query("damage")? {
			None => None,
			Some(node) => Some(DamageRoll::from_kdl(
				node,
				&mut ValueIdx::default(),
				system,
			)?),
		};
		Ok(Self {
			kind,
			check,
			area_of_effect,
			damage,
		})
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttackKind {
	Melee,
	Ranged,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackKindValue {
	Melee {
		reach: u32,
	},
	Ranged {
		short_dist: u32,
		long_dist: u32,
		kind: Option<RangeKind>,
	},
}
impl AttackKindValue {
	pub fn kind(&self) -> AttackKind {
		match self {
			Self::Melee { .. } => AttackKind::Melee,
			Self::Ranged { .. } => AttackKind::Ranged,
		}
	}
}
impl FromKDL<DnD5e> for AttackKindValue {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"Melee" => Ok(Self::Melee {
				reach: node.get_i64_opt("reach")?.unwrap_or(5) as u32,
			}),
			"Ranged" => {
				let short_dist = node.get_i64(value_idx.next())? as u32;
				let long_dist = node.get_i64(value_idx.next())? as u32;
				let kind = match node.get_str_opt("kind")? {
					None => None,
					Some(str) => Some(RangeKind::from_str(str)?),
				};
				Ok(Self::Ranged {
					short_dist,
					long_dist,
					kind,
				})
			}
			name => Err(GeneralError(format!(
				"Invalid attack kind {name:?}, expected Melee or Ranged."
			))
			.into()),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RangeKind {
	OnlySelf,
	Touch,
	Bounded, // abide by the short/long dist range
	Sight,
	Unlimited,
}
impl ToString for RangeKind {
	fn to_string(&self) -> String {
		match self {
			Self::OnlySelf => "Self",
			Self::Touch => "Touch",
			Self::Bounded => "Bounded",
			Self::Sight => "Sight",
			Self::Unlimited => "Unlimited",
		}
		.to_owned()
	}
}
impl FromStr for RangeKind {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Self" => Ok(Self::OnlySelf),
			"Touch" => Ok(Self::Touch),
			"Bounded" => Ok(Self::Bounded),
			"Sight" => Ok(Self::Sight),
			"Unlimited" => Ok(Self::Unlimited),
			_ => Err(GeneralError(format!(
				"Invalid kind of range {s:?}, expected Self, Touch, Bounded, Sight, or Unlimited."
			))),
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum AttackCheckKind {
	AttackRoll {
		ability: Ability,
		proficient: Value<bool>,
	},
	SavingThrow {
		base: i32,
		dc_ability: Option<Ability>,
		proficient: bool,
		save_ability: Ability,
	},
}

impl crate::utility::TraitEq for AttackCheckKind {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for AttackCheckKind {
	type Context = Character;
	type Item = i32;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::AttackRoll {
				ability,
				proficient,
			} => {
				let proficient = proficient.evaluate(state);
				state.ability_modifier(*ability, Some(proficient.into()))
			}
			Self::SavingThrow {
				base,
				dc_ability,
				proficient,
				save_ability: _,
			} => {
				let ability_bonus = dc_ability
					.as_ref()
					.map(|ability| state.ability_score(*ability).0.modifier())
					.unwrap_or_default();
				let prof_bonus = proficient
					.then(|| state.proficiency_bonus())
					.unwrap_or_default();
				*base + ability_bonus + prof_bonus
			}
		}
	}
}

impl FromKDL<DnD5e> for AttackCheckKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"AttackRoll" => {
				let ability = Ability::from_str(node.get_str(value_idx.next())?)?;
				let proficient = match (node.get_bool_opt("proficient")?, node.query("proficient")?)
				{
					(None, None) => Value::Fixed(false),
					(Some(prof), None) => Value::Fixed(prof),
					(_, Some(node)) => {
						let mut value_idx = ValueIdx::default();
						Value::from_kdl(
							node,
							node.entry_req(value_idx.next())?,
							&mut value_idx,
							system,
							|value| Ok(value.as_bool()),
						)?
					}
				};
				Ok(Self::AttackRoll {
					ability,
					proficient,
				})
			}
			"SavingThrow" => {
				// TODO: The difficulty class should be its own struct (which impls evaluator)
				let (base, dc_ability, proficient) = {
					let node = node.query_req("difficulty_class")?;
					let mut value_idx = ValueIdx::default();
					let base = node.get_i64(value_idx.next())? as i32;
					let ability = match node.query_str_opt("ability_bonus", 0)? {
						None => None,
						Some(str) => Some(Ability::from_str(str)?),
					};
					let proficient = node
						.query_bool_opt("proficiency_bonus", 0)?
						.unwrap_or(false);
					(base, ability, proficient)
				};
				let save_ability = Ability::from_str(node.query_str("save_ability", 0)?)?;
				Ok(Self::SavingThrow {
					base,
					dc_ability,
					proficient,
					save_ability,
				})
			}
			name => Err(GeneralError(format!(
				"Invalid attack check {name:?}, expected AttackRoll or SavingThrow"
			))
			.into()),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AreaOfEffect {
	Cone { length: u32 },
	Cube { size: u32 },
	Cylinder { radius: u32, height: u32 },
	Line { width: u32, length: u32 },
	Sphere { radius: u32 },
}

impl FromKDL<DnD5e> for AreaOfEffect {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"Cone" => Ok(Self::Cone {
				length: node.get_i64("length")? as u32,
			}),
			"Cube" => Ok(Self::Cube {
				size: node.get_i64("size")? as u32,
			}),
			"Cylinder" => Ok(Self::Cylinder {
				radius: node.get_i64("radius")? as u32,
				height: node.get_i64("height")? as u32,
			}),
			"Line" => Ok(Self::Line {
				width: node.get_i64("width")? as u32,
				length: node.get_i64("length")? as u32,
			}),
			"Sphere" => Ok(Self::Sphere {
				radius: node.get_i64("radius")? as u32,
			}),
			name => Err(GeneralError(format!(
				"Invalid area of effect {name:?}, \
				expected Cone, Cube, Cylinder, Line, or Sphere"
			))
			.into()),
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct DamageRoll {
	// TODO: Implement damage which scales according to some scalar (usually class, character, or spell level)
	pub roll: Option<Roll>,
	pub base_bonus: i32,
	pub damage_type: DamageType,
	// generated (see BonusDamage mutator)
	pub additional_bonuses: Vec<(i32, PathBuf)>,
}
impl FromKDL<DnD5e> for DamageRoll {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let roll = match node.query_str_opt("roll", 0)? {
			None => None,
			Some(str) => Some(Roll::from_str(str)?),
		};
		let base_bonus = node.get_i64_opt("base")?.unwrap_or(0) as i32;
		let damage_type = DamageType::from_str(node.query_str("damage_type", 0)?)?;
		Ok(Self {
			roll,
			base_bonus,
			damage_type,
			additional_bonuses: Vec::new(),
		})
	}
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DamageType {
	Acid,
	Bludgeoning,
	Cold,
	#[default]
	Fire,
	Force,
	Lightning,
	Necrotic,
	Piercing,
	Poison,
	Psychic,
	Radiant,
	Slashing,
	Thunder,
}
impl FromStr for DamageType {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Acid" => Ok(Self::Acid),
			"Bludgeoning" => Ok(Self::Bludgeoning),
			"Cold" => Ok(Self::Cold),
			"Fire" => Ok(Self::Fire),
			"Force" => Ok(Self::Force),
			"Lightning" => Ok(Self::Lightning),
			"Necrotic" => Ok(Self::Necrotic),
			"Piercing" => Ok(Self::Piercing),
			"Poison" => Ok(Self::Poison),
			"Psychic" => Ok(Self::Psychic),
			"Radiant" => Ok(Self::Radiant),
			"Slashing" => Ok(Self::Slashing),
			"Thunder" => Ok(Self::Thunder),
			_ => Err(GeneralError(format!("Invalid damage type {s:?}")).into()),
		}
	}
}
impl DamageType {
	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Acid => "Acid",
			Self::Bludgeoning => "Bludgeoning",
			Self::Cold => "Cold",
			Self::Fire => "Fire",
			Self::Force => "Force",
			Self::Lightning => "Lightning",
			Self::Necrotic => "Necrotic",
			Self::Piercing => "Piercing",
			Self::Poison => "Poison",
			Self::Psychic => "Psychic",
			Self::Radiant => "Radiant",
			Self::Slashing => "Slashing",
			Self::Thunder => "Thunder",
		}
	}

	pub fn description(&self) -> &'static str {
		match self {
			Self::Acid => "The corrosive spray of an adult black dragon's breath and the dissolving \
			enzymes secreted by a black pudding deal acid damage.",
			Self::Bludgeoning => "Blunt force attacks--hammers, falling, constriction, \
			and the like--deal bludgeoning damage.",
			Self::Cold => "The infernal chill radiating from an ice devil's spear and the frigid blast \
			of a young white dragon's breath deal cold damage.",
			Self::Fire => "Ancient red dragons breathe fire, and many spells conjure flames to deal fire damage.",
			Self::Force => "Force is pure magical energy focused into a damaging form. \
			Most effects that deal force damage are spells, including magic missile and spiritual weapon.",
			Self::Lightning => "A lightning bolt spell and a blue dragon wyrmling's breath deal lightning damage.",
			Self::Necrotic => "Necrotic damage, dealt by certain undead and a spell such \
			as chill touch, withers matter and even the soul.",
			Self::Piercing => "Puncturing and impaling attacks, including spears and \
			monsters' bites, deal piercing damage.",
			Self::Poison => "Venomous stings and the toxic gas of an adult green dragon's breath deal poison damage.",
			Self::Psychic => "Mental abilities such as a psionic blast deal psychic damage.",
			Self::Radiant => "Radiant damage, dealt by a cleric's flame strike spell or an angel's \
			smiting weapon, sears the flesh like fire and overloads the spirit with power.",
			Self::Slashing => "Swords, axes, and monsters' claws deal slashing damage.",
			Self::Thunder => "A concussive burst of sound, such as the effect of the thunderwave spell, deals thunder damage.",
		}
	}
}
