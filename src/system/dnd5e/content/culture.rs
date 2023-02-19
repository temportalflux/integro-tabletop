use super::{lineage::changeling, upbringing::incognito};
use crate::system::dnd5e::character::Culture;

pub fn changeling() -> Culture {
	Culture {
		lineages: [changeling::shapechanger(), changeling::voice_changer()],
		upbringing: incognito(),
	}
}
