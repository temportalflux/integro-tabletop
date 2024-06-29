use crate::system::dnd5e::data::roll::RollSet;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct SizeFormula {
	/// Equation used to calcuate height = base + bonus
	pub height: HeightFormula,
	/// Equation used to calculate weight = base + (height bonus * multiplier) + bonus
	pub weight: WeightFormula,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct HeightFormula {
	pub base: u32,
	pub bonus: RollSet,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct WeightFormula {
	pub base: u32,
	pub multiplier: RollSet,
	pub bonus: RollSet,
}

impl SizeFormula {
	pub fn get_random(&self, rand: &mut impl rand::Rng) -> (u32, u32) {
		let height_mod = self.height.bonus.roll(rand);
		let height = self.height.base.saturating_add_signed(height_mod);
		let weight_mult = self.weight.multiplier.roll(rand) * height_mod;
		let weight = self.weight.base.saturating_add_signed(weight_mult + self.weight.bonus.roll(rand));
		(height, weight)
	}

	pub fn min_height(&self) -> u32 {
		self.height.base.saturating_add_signed(self.height.bonus.min())
	}

	pub fn max_height(&self) -> u32 {
		self.height.base.saturating_add_signed(self.height.bonus.max())
	}

	pub fn min_weight(&self) -> u32 {
		self.weight
			.base
			.saturating_add_signed(self.weight.bonus.min() + (self.height.bonus.min() * self.weight.multiplier.min()))
	}

	pub fn max_weight(&self) -> u32 {
		self.weight
			.base
			.saturating_add_signed(self.weight.bonus.max() + (self.height.bonus.max() * self.weight.multiplier.max()))
	}
}
