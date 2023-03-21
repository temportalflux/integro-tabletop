use enum_map::EnumMap;
mod kind;
use itertools::Itertools;
pub use kind::*;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Wallet(EnumMap<CurrencyKind, u64>);
impl std::ops::Index<CurrencyKind> for Wallet {
	type Output = u64;

	fn index(&self, index: CurrencyKind) -> &Self::Output {
		&self.0[index]
	}
}
impl std::ops::IndexMut<CurrencyKind> for Wallet {
	fn index_mut(&mut self, index: CurrencyKind) -> &mut Self::Output {
		&mut self.0[index]
	}
}
impl<const N: usize> From<[(u64, CurrencyKind); N]> for Wallet {
	fn from(values: [(u64, CurrencyKind); N]) -> Self {
		let mut wallet = Self::default();
		for (amt, kind) in values {
			wallet[kind] += amt;
		}
		wallet
	}
}
impl From<u64> for Wallet {
	fn from(mut total: u64) -> Self {
		let mut pouch = Self::default();
		for kind in CurrencyKind::all().sorted().rev() {
			if kind == CurrencyKind::Electrum {
				// we are biased and do not convert into this type
				continue;
			}
			if total >= kind.multiplier() {
				let amt = total / kind.multiplier();
				total -= amt * kind.multiplier();
				pouch.0[kind] = amt;
			}
		}
		pouch
	}
}
impl std::ops::Add for Wallet {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let mut output = self.clone();
		for (kind, amt) in rhs.0 {
			output.0[kind] += amt;
		}
		output
	}
}
impl std::ops::AddAssign for Wallet {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}
impl Wallet {
	pub fn total_value(&self) -> u64 {
		self.0
			.iter()
			.map(|(kind, amt)| amt * kind.multiplier())
			.sum::<u64>()
	}

	pub fn is_empty(&self) -> bool {
		*self == Self::default()
	}

	pub fn contains(&self, other: &Wallet, auto_exchange: bool) -> bool {
		// if we can exchange, then we will always be able to cover other
		// as long as we have a greater total value
		if auto_exchange {
			return self.total_value() >= other.total_value();
		}
		// if not exchanging, we only have enough to cover other
		// if we have enough in each discrete category
		for (kind, amt) in other.0 {
			if self.0[kind] < amt {
				return false;
			}
		}
		true
	}

	pub fn remove(&mut self, amounts: Self, auto_exchange: bool) {
		if !self.contains(&amounts, auto_exchange) {
			panic!("Wallet must contain all of amounts before calling remove.");
		}
		// the amount remaining after removing discrete amounts.
		// will require auto_exchange in order to complete.
		let mut remainder = Wallet::default();
		// iterate over all amounts to remove, Copper -> Platinum
		for (kind, amt) in amounts.0 {
			// if we have enough, then this is easy and we just subtract
			if self[kind] >= amt {
				self[kind] -= amt;
			}
			// if we dont have enough, then we will need to exchange some amount
			else {
				// save the remainder in the temporary holdings
				remainder[kind] += amt - self[kind];
				// output remains empty, because we've consumed it all
				self[kind] = 0;
			}
		}
		assert!(remainder.is_empty() || auto_exchange);

		// While `remainder` could be just a number, we dont want to consume
		// smaller currencies when exchanging, so we need to know
		// how much of each discrete unit is remaining.
		// The actual algorithm to consume the remainder is the inner loop,
		// if remainder were just a number, we could just use the inner loop
		// without filtering the currency kind that can be considered.
		for (kind, remainder_amt) in remainder.0 {
			if remainder_amt <= 0 {
				continue;
			}
			let mut remainder = remainder_amt * kind.multiplier();

			for (target_kind, amount) in &mut self.0 {
				// we dont want to consider this entry if either:
				// a. the target kind is smaller than the remainder kind (e.g. Copper < Gold)
				// b. the amount in this unit is empty
				if target_kind < kind || *amount <= 0 {
					continue;
				}

				let total = *amount * target_kind.multiplier();
				// there isn't enough here to satisfy remainder,
				// so consume it all and remove that sum from the remainder
				if total < remainder {
					*amount = 0;
					remainder -= total;
				}
				// total >= remainder
				// the total in this unit can consume all of remainder
				else {
					// The new `remainder` is the amount resulting after removing the remaining cost.
					// This is the amount that was left in `total`,
					// but definitely is not divisible by the current unit.
					// i.e. `(total - remainder) % kind.multiplier() != 0`
					let mut remainder = total - remainder;

					// Re-add the remainder as a new wallet, converting the total amt into discrete units.
					*amount = remainder / target_kind.multiplier();
					remainder = remainder % target_kind.multiplier();
					*self += Self::from(remainder);

					// We've consumed this remainder, move onto the next one in the total currencies remaining
					break;
				}
			}
		}
	}

	/// converts larger amounts of smaller currencies into their largest possible currency
	pub fn normalize(&mut self) {
		*self = Self::from(self.total_value());
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn empty() {
		let wallet = Wallet::default();
		assert_eq!(wallet.total_value(), 0);
		assert!(wallet.is_empty());
		assert_eq!(wallet[CurrencyKind::Copper], 0);
		assert_eq!(wallet[CurrencyKind::Silver], 0);
		assert_eq!(wallet[CurrencyKind::Electrum], 0);
		assert_eq!(wallet[CurrencyKind::Gold], 0);
		assert_eq!(wallet[CurrencyKind::Platinum], 0);
	}

	#[test]
	fn from_u64() {
		let wallet = Wallet::from(15753);
		assert_eq!(wallet.total_value(), 15753);
		assert_eq!(wallet[CurrencyKind::Copper], 3);
		assert_eq!(wallet[CurrencyKind::Silver], 5);
		assert_eq!(wallet[CurrencyKind::Electrum], 0);
		assert_eq!(wallet[CurrencyKind::Gold], 7);
		assert_eq!(wallet[CurrencyKind::Platinum], 15);
	}

	#[test]
	fn from_discrete() {
		let wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		assert_eq!(wallet.total_value(), 13173);
		assert_eq!(wallet[CurrencyKind::Copper], 53);
		assert_eq!(wallet[CurrencyKind::Silver], 247);
		assert_eq!(wallet[CurrencyKind::Electrum], 1);
		assert_eq!(wallet[CurrencyKind::Gold], 86);
		assert_eq!(wallet[CurrencyKind::Platinum], 2);
	}

	#[test]
	fn add_empty_exchanged() {
		let mut wallet = Wallet::default();
		wallet += Wallet::from(15753);
		assert_eq!(wallet.total_value(), 15753);
		assert_eq!(wallet[CurrencyKind::Copper], 3);
		assert_eq!(wallet[CurrencyKind::Silver], 5);
		assert_eq!(wallet[CurrencyKind::Electrum], 0);
		assert_eq!(wallet[CurrencyKind::Gold], 7);
		assert_eq!(wallet[CurrencyKind::Platinum], 15);
	}

	#[test]
	fn add_some_discrete() {
		let mut wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		wallet += Wallet::from([
			(12, CurrencyKind::Copper),
			(37, CurrencyKind::Silver),
			(81, CurrencyKind::Electrum),
			(74, CurrencyKind::Gold),
			(6, CurrencyKind::Platinum),
		]);
		assert_eq!(wallet.total_value(), 31005);
		assert_eq!(wallet[CurrencyKind::Copper], 65);
		assert_eq!(wallet[CurrencyKind::Silver], 284);
		assert_eq!(wallet[CurrencyKind::Electrum], 82);
		assert_eq!(wallet[CurrencyKind::Gold], 160);
		assert_eq!(wallet[CurrencyKind::Platinum], 8);
	}

	#[test]
	fn add_some_exchanged() {
		let mut wallet = Wallet::from(7986);
		wallet += Wallet::from(5347);
		assert_eq!(wallet.total_value(), 13333);
		assert_eq!(wallet[CurrencyKind::Copper], 13);
		assert_eq!(wallet[CurrencyKind::Silver], 12);
		assert_eq!(wallet[CurrencyKind::Electrum], 0);
		assert_eq!(wallet[CurrencyKind::Gold], 12);
		assert_eq!(wallet[CurrencyKind::Platinum], 12);
	}

	#[test]
	fn contains_less_equal() {
		let wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		let other = Wallet::from([
			(5, CurrencyKind::Copper),
			(20, CurrencyKind::Silver),
		]);
		// auto-exchange shouldnt matter here
		assert!(wallet.contains(&other, false));
		assert!(wallet.contains(&other, true));
		assert!(wallet.contains(&wallet, false));
		assert!(wallet.contains(&wallet, true));
	}
	
	#[test]
	fn contains_larger_discrete() {
		let wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		let other = Wallet::from([
			(60, CurrencyKind::Copper),
			(2, CurrencyKind::Electrum),
		]);
		// only contains if exchange is enabled
		assert!(!wallet.contains(&other, false));
		assert!(wallet.contains(&other, true));
	}
	
	#[test]
	fn remove_no_exchange() {
		let mut wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		let other = Wallet::from([
			(50, CurrencyKind::Copper),
			(30, CurrencyKind::Gold),
		]);
		wallet.remove(other, false);
		assert_eq!(wallet.total_value(), 10123);
		assert_eq!(wallet[CurrencyKind::Copper], 3);
		assert_eq!(wallet[CurrencyKind::Silver], 247);
		assert_eq!(wallet[CurrencyKind::Electrum], 1);
		assert_eq!(wallet[CurrencyKind::Gold], 56);
		assert_eq!(wallet[CurrencyKind::Platinum], 2);
	}

	#[test]
	fn remove_exchange() {
		let mut wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		let other = Wallet::from([
			(60, CurrencyKind::Copper),
			(3, CurrencyKind::Electrum),
			(90, CurrencyKind::Gold),
		]);
		wallet.remove(other, true);
		assert_eq!(wallet.total_value(), 3963);
		assert_eq!(wallet[CurrencyKind::Copper], 3);
		assert_eq!(wallet[CurrencyKind::Silver], 246);
		assert_eq!(wallet[CurrencyKind::Electrum], 0);
		assert_eq!(wallet[CurrencyKind::Gold], 5);
		assert_eq!(wallet[CurrencyKind::Platinum], 1);
	}

	#[test]
	fn normalize() {
		let mut wallet = Wallet::from([
			(53, CurrencyKind::Copper),
			(247, CurrencyKind::Silver),
			(1, CurrencyKind::Electrum),
			(86, CurrencyKind::Gold),
			(2, CurrencyKind::Platinum),
		]);
		wallet.normalize();
		assert_eq!(wallet.total_value(), 13173);
		assert_eq!(wallet[CurrencyKind::Copper], 3);
		assert_eq!(wallet[CurrencyKind::Silver], 7);
		assert_eq!(wallet[CurrencyKind::Electrum], 0);
		assert_eq!(wallet[CurrencyKind::Gold], 1);
		assert_eq!(wallet[CurrencyKind::Platinum], 13);
	}
}
