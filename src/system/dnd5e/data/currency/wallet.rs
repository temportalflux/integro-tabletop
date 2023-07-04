use super::Kind;
use crate::kdl_ext::{AsKdl, FromKDL, NodeBuilder};
use enum_map::EnumMap;
use itertools::Itertools;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Wallet(EnumMap<Kind, u64>);

impl std::ops::Index<Kind> for Wallet {
	type Output = u64;

	fn index(&self, index: Kind) -> &Self::Output {
		&self.0[index]
	}
}

impl std::ops::IndexMut<Kind> for Wallet {
	fn index_mut(&mut self, index: Kind) -> &mut Self::Output {
		&mut self.0[index]
	}
}

impl<const N: usize> From<[(u64, Kind); N]> for Wallet {
	fn from(values: [(u64, Kind); N]) -> Self {
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
		for kind in Kind::all().sorted().rev() {
			if kind == Kind::Electrum {
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

impl std::ops::Mul<u64> for Wallet {
	type Output = Self;

	fn mul(mut self, rhs: u64) -> Self::Output {
		for (_, amt) in &mut self.0 {
			*amt *= rhs;
		}
		self
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

impl FromKDL for Wallet {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut wallet = Self::default();
		if !node.entries().is_empty() {
			wallet += Self::from_row(node)?;
		}
		for mut node in &mut node.query_all("scope() > item")? {
			wallet += Self::from_row(&mut node)?;
		}
		Ok(wallet)
	}
}
impl AsKdl for Wallet {
	fn as_kdl(&self) -> NodeBuilder {
		let mut rows = Vec::new();
		for (kind, amt) in &self.0 {
			if *amt <= 0 {
				continue;
			}
			rows.push(NodeBuilder::default().with_entry(*amt as i64).with_entry({
				let mut currency = kdl::KdlEntry::new(kind.to_string());
				currency.set_ty("Currency");
				currency
			}));
		}
		if rows.is_empty() {
			NodeBuilder::default()
		} else if rows.len() == 1 {
			rows.into_iter().next().unwrap()
		} else {
			let mut node = NodeBuilder::default();
			for row in rows {
				node.push_child(row.build("item"));
			}
			node
		}
	}
}
impl Wallet {
	fn from_row<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let amt = node.next_i64_req()? as u64;
		let kind = node.next_str_req_t::<Kind>()?;
		Ok(Wallet::from([(amt, kind)]))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "wallet";

		#[test]
		fn single() -> anyhow::Result<()> {
			let doc = "wallet 1 (Currency)\"Copper\"";
			let data = Wallet::from([(1, Kind::Copper)]);
			assert_eq_fromkdl!(Wallet, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn multiple() -> anyhow::Result<()> {
			let doc = "
				|wallet {
				|    item 5 (Currency)\"Copper\"
				|    item 20 (Currency)\"Silver\"
				|    item 3 (Currency)\"Gold\"
				|}
			";
			let data = Wallet::from([(5, Kind::Copper), (20, Kind::Silver), (3, Kind::Gold)]);
			assert_eq_fromkdl!(Wallet, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn duplicates() -> anyhow::Result<()> {
			let doc_in = "
				|wallet {
				|    item 5 (Currency)\"Copper\"
				|    item 10 (Currency)\"Silver\"
				|    item 3 (Currency)\"Gold\"
				|    item 5 (Currency)\"Silver\"
				|    item 20 (Currency)\"Copper\"
				|}
			";
			let doc_out = "
				|wallet {
				|    item 25 (Currency)\"Copper\"
				|    item 15 (Currency)\"Silver\"
				|    item 3 (Currency)\"Gold\"
				|}
			";
			let data = Wallet::from([(25, Kind::Copper), (15, Kind::Silver), (3, Kind::Gold)]);
			assert_eq_fromkdl!(Wallet, doc_in, data);
			assert_eq_askdl!(&data, doc_out);
			Ok(())
		}
	}

	#[test]
	fn empty() {
		let wallet = Wallet::default();
		assert_eq!(wallet.total_value(), 0);
		assert!(wallet.is_empty());
		assert_eq!(wallet[Kind::Copper], 0);
		assert_eq!(wallet[Kind::Silver], 0);
		assert_eq!(wallet[Kind::Electrum], 0);
		assert_eq!(wallet[Kind::Gold], 0);
		assert_eq!(wallet[Kind::Platinum], 0);
	}

	#[test]
	fn from_u64() {
		let wallet = Wallet::from(15753);
		assert_eq!(wallet.total_value(), 15753);
		assert_eq!(wallet[Kind::Copper], 3);
		assert_eq!(wallet[Kind::Silver], 5);
		assert_eq!(wallet[Kind::Electrum], 0);
		assert_eq!(wallet[Kind::Gold], 7);
		assert_eq!(wallet[Kind::Platinum], 15);
	}

	#[test]
	fn from_discrete() {
		let wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		assert_eq!(wallet.total_value(), 13173);
		assert_eq!(wallet[Kind::Copper], 53);
		assert_eq!(wallet[Kind::Silver], 247);
		assert_eq!(wallet[Kind::Electrum], 1);
		assert_eq!(wallet[Kind::Gold], 86);
		assert_eq!(wallet[Kind::Platinum], 2);
	}

	#[test]
	fn add_empty_exchanged() {
		let mut wallet = Wallet::default();
		wallet += Wallet::from(15753);
		assert_eq!(wallet.total_value(), 15753);
		assert_eq!(wallet[Kind::Copper], 3);
		assert_eq!(wallet[Kind::Silver], 5);
		assert_eq!(wallet[Kind::Electrum], 0);
		assert_eq!(wallet[Kind::Gold], 7);
		assert_eq!(wallet[Kind::Platinum], 15);
	}

	#[test]
	fn add_some_discrete() {
		let mut wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		wallet += Wallet::from([
			(12, Kind::Copper),
			(37, Kind::Silver),
			(81, Kind::Electrum),
			(74, Kind::Gold),
			(6, Kind::Platinum),
		]);
		assert_eq!(wallet.total_value(), 31005);
		assert_eq!(wallet[Kind::Copper], 65);
		assert_eq!(wallet[Kind::Silver], 284);
		assert_eq!(wallet[Kind::Electrum], 82);
		assert_eq!(wallet[Kind::Gold], 160);
		assert_eq!(wallet[Kind::Platinum], 8);
	}

	#[test]
	fn add_some_exchanged() {
		let mut wallet = Wallet::from(7986);
		wallet += Wallet::from(5347);
		assert_eq!(wallet.total_value(), 13333);
		assert_eq!(wallet[Kind::Copper], 13);
		assert_eq!(wallet[Kind::Silver], 12);
		assert_eq!(wallet[Kind::Electrum], 0);
		assert_eq!(wallet[Kind::Gold], 12);
		assert_eq!(wallet[Kind::Platinum], 12);
	}

	#[test]
	fn contains_less_equal() {
		let wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		let other = Wallet::from([(5, Kind::Copper), (20, Kind::Silver)]);
		// auto-exchange shouldnt matter here
		assert!(wallet.contains(&other, false));
		assert!(wallet.contains(&other, true));
		assert!(wallet.contains(&wallet, false));
		assert!(wallet.contains(&wallet, true));
	}

	#[test]
	fn contains_larger_discrete() {
		let wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		let other = Wallet::from([(60, Kind::Copper), (2, Kind::Electrum)]);
		// only contains if exchange is enabled
		assert!(!wallet.contains(&other, false));
		assert!(wallet.contains(&other, true));
	}

	#[test]
	fn remove_no_exchange() {
		let mut wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		let other = Wallet::from([(50, Kind::Copper), (30, Kind::Gold)]);
		wallet.remove(other, false);
		assert_eq!(wallet.total_value(), 10123);
		assert_eq!(wallet[Kind::Copper], 3);
		assert_eq!(wallet[Kind::Silver], 247);
		assert_eq!(wallet[Kind::Electrum], 1);
		assert_eq!(wallet[Kind::Gold], 56);
		assert_eq!(wallet[Kind::Platinum], 2);
	}

	#[test]
	fn remove_exchange() {
		let mut wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		let other = Wallet::from([(60, Kind::Copper), (3, Kind::Electrum), (90, Kind::Gold)]);
		wallet.remove(other, true);
		assert_eq!(wallet.total_value(), 3963);
		assert_eq!(wallet[Kind::Copper], 3);
		assert_eq!(wallet[Kind::Silver], 246);
		assert_eq!(wallet[Kind::Electrum], 0);
		assert_eq!(wallet[Kind::Gold], 5);
		assert_eq!(wallet[Kind::Platinum], 1);
	}

	#[test]
	fn normalize() {
		let mut wallet = Wallet::from([
			(53, Kind::Copper),
			(247, Kind::Silver),
			(1, Kind::Electrum),
			(86, Kind::Gold),
			(2, Kind::Platinum),
		]);
		wallet.normalize();
		assert_eq!(wallet.total_value(), 13173);
		assert_eq!(wallet[Kind::Copper], 3);
		assert_eq!(wallet[Kind::Silver], 7);
		assert_eq!(wallet[Kind::Electrum], 0);
		assert_eq!(wallet[Kind::Gold], 1);
		assert_eq!(wallet[Kind::Platinum], 13);
	}
}
