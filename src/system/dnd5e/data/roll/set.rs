use super::{Die, Roll};
use enum_map::{Enum, EnumMap};

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct RollSet(EnumMap<Die, u32>, u32);

impl From<Roll> for RollSet {
	fn from(value: Roll) -> Self {
		let mut set = Self::default();
		set.push(value);
		set
	}
}

impl From<u32> for RollSet {
	fn from(value: u32) -> Self {
		Self::from(Roll::from(value))
	}
}

impl RollSet {
	pub fn multiple(roll: &Roll, amount: u32) -> Self {
		let mut set = Self::default();
		match &roll.die {
			None => set.1 += roll.amount * amount,
			Some(die) => {
				set.0[*die] += roll.amount * amount;
			}
		}
		set
	}

	pub fn push(&mut self, roll: Roll) {
		match roll.die {
			None => self.1 += roll.amount,
			Some(die) => {
				self.0[die] += roll.amount;
			}
		}
	}

	pub fn remove(&mut self, roll: Roll) {
		match roll.die {
			None => self.1 = self.1.saturating_sub(roll.amount),
			Some(die) => {
				self.0[die] = self.0[die].saturating_sub(roll.amount);
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

	pub fn take_flat_bonus(&mut self) -> u32 {
		let out = self.1;
		self.1 = 0;
		out
	}

	pub fn rolls(&self) -> Vec<Roll> {
		let mut rolls = Vec::with_capacity(Die::LENGTH + 1);
		for (die, amt) in &self.0 {
			if *amt == 0 {
				continue;
			}
			rolls.push(Roll::from((*amt, die)));
		}
		if self.1 > 0 {
			rolls.push(Roll::from(self.1));
		}
		rolls
	}

	pub fn min(&self) -> u32 {
		let mut value = self.1;
		for (_die, amt) in &self.0 {
			if *amt > 0 {
				value += *amt;
			}
		}
		value
	}

	pub fn max(&self) -> u32 {
		let mut value = self.1;
		for (die, amt) in &self.0 {
			if *amt > 0 {
				value += *amt * die.value();
			}
		}
		value
	}

	pub fn as_nonzero_string(&self) -> Option<String> {
		let mut roll_strs = self
			.rolls()
			.iter()
			.filter_map(Roll::as_nonzero_string)
			.collect::<Vec<_>>();
		if self.1 > 0 {
			roll_strs.push(self.1.to_string());
		}
		(!roll_strs.is_empty()).then(|| roll_strs.join(" + "))
	}

	pub fn roll(&self, rand: &mut impl rand::Rng) -> u32 {
		let mut value = self.1;
		for (die, amt) in &self.0 {
			value += die.roll(rand, *amt);
		}
		value
	}
}
