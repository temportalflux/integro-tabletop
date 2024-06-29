use super::{Die, Roll};
use enum_map::EnumMap;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct RollSet(EnumMap<Die, u32>, i32);

impl From<Roll> for RollSet {
	fn from(value: Roll) -> Self {
		let mut set = Self::default();
		set.push(value);
		set
	}
}

impl FromIterator<Roll> for RollSet {
	fn from_iter<T: IntoIterator<Item = Roll>>(iter: T) -> Self {
		let mut set = Self::default();
		for roll in iter {
			set.push(roll);
		}
		set
	}
}

impl From<i32> for RollSet {
	fn from(value: i32) -> Self {
		Self::from(Roll::from(value))
	}
}

impl RollSet {
	pub fn multiple(roll: &Roll, amount: u32) -> Self {
		let mut set = Self::default();
		match &roll.die {
			None => set.1 += roll.amount * amount as i32,
			Some(die) => {
				set.0[*die] += roll.amount as u32 * amount;
			}
		}
		set
	}

	pub fn push(&mut self, roll: Roll) {
		match roll.die {
			None => self.1 += roll.amount,
			Some(die) => {
				self.0[die] += roll.amount as u32;
			}
		}
	}

	pub fn remove(&mut self, roll: Roll) {
		match roll.die {
			None => self.1 = self.1.saturating_sub(roll.amount),
			Some(die) => {
				self.0[die] = self.0[die].saturating_sub(roll.amount as u32);
			}
		}
	}

	pub fn extend(&mut self, set: RollSet) {
		for (die, amt) in set.0 {
			self.0[die] += amt;
		}
		self.1 += set.1;
	}

	pub fn is_empty(&self) -> bool {
		if self.1 > 0 {
			return false;
		}
		for (_, amt) in &self.0 {
			if *amt > 0 {
				return false;
			}
		}
		true
	}

	pub fn take_flat_bonus(&mut self) -> i32 {
		let out = self.1;
		self.1 = 0;
		out
	}

	pub fn iter_rolls(&self) -> impl Iterator<Item = Roll> + '_ {
		let iter = self.0.iter();
		let iter = iter.filter_map(|(die, amt)| if *amt == 0 { None } else { Some(Roll::from((*amt, die))) });
		let fixed = (self.1 != 0).then(|| Roll::from(self.1));
		iter.chain(fixed.into_iter())
	}

	pub fn rolls(&self) -> Vec<Roll> {
		self.iter_rolls().collect()
	}

	pub fn min(&self) -> i32 {
		let mut value = self.1;
		for (_die, amt) in &self.0 {
			if *amt > 0 {
				value += *amt as i32;
			}
		}
		value
	}

	pub fn max(&self) -> i32 {
		let mut value = self.1;
		for (die, amt) in &self.0 {
			if *amt > 0 {
				value += (*amt * die.value()) as i32;
			}
		}
		value
	}

	pub fn roll(&self, rand: &mut impl rand::Rng) -> i32 {
		let mut value = self.1;
		for (die, amt) in &self.0 {
			value += die.roll(rand, *amt) as i32;
		}
		value
	}
}

impl std::fmt::Display for RollSet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut first = true;
		for roll in self.iter_rolls() {
			if roll.amount == 0 {
				continue;
			}

			let roll_str = roll.to_string();
			if first {
				write!(f, "{roll_str}")?;
				first = false;
				continue;
			}

			if let Some(stripped) = roll_str.strip_prefix("-") {
				write!(f, " - {stripped}")?;
			}
			else {
				write!(f, " + {roll_str}")?;
			}
		}
		Ok(())
	}
}
