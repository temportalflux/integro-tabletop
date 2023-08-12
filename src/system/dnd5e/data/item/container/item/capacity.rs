#[derive(Clone, PartialEq, Default, Debug)]
pub struct Capacity {
	pub count: Option<usize>,
	// Unit: pounds (lbs)
	pub weight: Option<f64>,
	// Unit: cubic feet
	pub volume: Option<f64>,
}
