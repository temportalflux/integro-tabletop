use std::str::FromStr;

use crate::GeneralError;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
	None,
	HalfDown,
	HalfUp,
	Full,
	Double,
}

impl Default for Level {
	fn default() -> Self {
		Self::None
	}
}
impl From<bool> for Level {
	fn from(value: bool) -> Self {
		match value {
			true => Self::Full,
			false => Self::None,
		}
	}
}

impl Level {
	pub fn as_display_name(&self) -> &'static str {
		match self {
			Self::None => "Not Proficient",
			Self::HalfDown => "Half Proficient (rounded down)",
			Self::HalfUp => "Half Proficient (rounded up)",
			Self::Full => "Proficient",
			Self::Double => "Expertise",
		}
	}
}

impl ToString for Level {
	fn to_string(&self) -> String {
		match self {
			Self::None => "None",
			Self::HalfDown => "HalfDown",
			Self::HalfUp => "HalfUp",
			Self::Full => "Full",
			Self::Double => "Double",
		}
		.to_owned()
	}
}

impl FromStr for Level {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"None" => Ok(Self::None),
			"HalfDown" => Ok(Self::HalfDown),
			"HalfUp" => Ok(Self::HalfUp),
			"Full" => Ok(Self::Full),
			"Double" => Ok(Self::Double),
			_ => Err(GeneralError(format!(
				"Invalid proficiency level {s:?}, expected None, Half, Full, or Double"
			))),
		}
	}
}

impl std::ops::Mul<i32> for Level {
	type Output = i32;

	fn mul(self, prof_bonus: i32) -> Self::Output {
		match self {
			Self::None => 0,
			Self::HalfDown => ((prof_bonus as f32) * 0.5).floor() as i32,
			Self::HalfUp => ((prof_bonus as f32) * 0.5).ceil() as i32,
			Self::Full => prof_bonus,
			Self::Double => prof_bonus * 2,
		}
	}
}

pub fn level_map() -> &'static [(usize, Option<usize>, i32)] {
	static MAP: [(usize, Option<usize>, i32); 5] = [
		(1, Some(4), 2),
		(5, Some(8), 3),
		(9, Some(12), 4),
		(13, Some(16), 5),
		(17, None, 6),
	];
	&MAP
}

pub fn proficiency_bonus(level: usize) -> i32 {
	level_map()
		.iter()
		.filter(|(min, _, _)| level >= *min)
		.filter(|(_, max, _)| match max {
			Some(max) => level <= *max,
			None => true,
		})
		.map(|(_, _, bonus)| *bonus)
		.next()
		.unwrap_or_default()
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn prof_map() {
		assert_eq!(proficiency_bonus(0), 0);
		assert_eq!(proficiency_bonus(1), 2);
		assert_eq!(proficiency_bonus(2), 2);
		assert_eq!(proficiency_bonus(3), 2);
		assert_eq!(proficiency_bonus(4), 2);
		assert_eq!(proficiency_bonus(5), 3);
		assert_eq!(proficiency_bonus(6), 3);
		assert_eq!(proficiency_bonus(7), 3);
		assert_eq!(proficiency_bonus(8), 3);
		assert_eq!(proficiency_bonus(9), 4);
		assert_eq!(proficiency_bonus(10), 4);
		assert_eq!(proficiency_bonus(11), 4);
		assert_eq!(proficiency_bonus(12), 4);
		assert_eq!(proficiency_bonus(13), 5);
		assert_eq!(proficiency_bonus(14), 5);
		assert_eq!(proficiency_bonus(15), 5);
		assert_eq!(proficiency_bonus(16), 5);
		assert_eq!(proficiency_bonus(17), 6);
		assert_eq!(proficiency_bonus(18), 6);
		assert_eq!(proficiency_bonus(19), 6);
		assert_eq!(proficiency_bonus(20), 6);
		assert_eq!(proficiency_bonus(21), 6);
	}
}
