use enum_map::Enum;
use enumset::EnumSetType;

#[derive(Debug, EnumSetType, Enum)]
pub enum Size {
	Tiny,
	Small,
	Medium,
	Large,
	Huge,
	Gargantuan,
}

impl ToString for Size {
	fn to_string(&self) -> String {
		match self {
			Self::Tiny => "Tiny",
			Self::Small => "Small",
			Self::Medium => "Medium",
			Self::Large => "Large",
			Self::Huge => "Huge",
			Self::Gargantuan => "Gargantuan",
		}
		.into()
	}
}

impl Size {
	pub fn description(&self) -> &'static str {
		match self {
			Size::Small => "Creatures less than 45 inches tall are Small sized. You control a 5 by 5 ft. space in combat. You can squeeze through Tiny spaces.",
			Size::Medium => "Creatures at least 45 inches tall are Medium sized. You control a 5 by 5 ft. space in combat. You can squeeze through Small spaces.",
			_ => "",
		}
	}
}
