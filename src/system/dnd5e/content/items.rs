use crate::system::dnd5e::data::{
	item::{equipment::Equipment, Item, ItemKind},
	mutator,
};

pub mod armor;
pub mod weapon;

pub fn travelers_clothes() -> Item {
	Item {
		name: "Traveler's Clothes".into(),
		description: Some(
			"This set of clothes could consist of boots, a wool skirt or breeches, \
		a sturdy belt, a shirt (perhaps with a vest or jacket), and an ample cloak with a hood."
				.into(),
		),
		weight: 4.0,
		worth: 200,
		..Default::default()
	}
}

pub fn goggles_of_night() -> Item {
	Item {
		name: "Goggles of Night".into(),
		description: Some(
			"While wearing these dark lenses, you have darkvision \
		out to a range of 60 feet. If you already have darkvision, wearing the \
		goggles increases its range by 60 feet."
				.into(),
		),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![mutator::AddMaxSense("Darkvision".into(), 60).into()],
			..Default::default()
		}),
		..Default::default()
	}
}
