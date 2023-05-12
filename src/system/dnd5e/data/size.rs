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
		}.into()
	}
}
