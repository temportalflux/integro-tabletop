use crate::system::dnd5e::data::roll::Roll;

pub trait DefaultLevelMap {
	fn default_for_level(level: usize) -> Option<Self>
	where
		Self: Sized;
}

impl DefaultLevelMap for u32 {
	fn default_for_level(level: usize) -> Option<Self> {
		Some(level as u32)
	}
}

impl DefaultLevelMap for Roll {
	fn default_for_level(_level: usize) -> Option<Self> {
		None
	}
}
