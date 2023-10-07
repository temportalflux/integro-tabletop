use crate::kdl_ext::NodeContext;
use crate::{system::dnd5e::data::roll::Roll, GeneralError};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub enum Property {
	Light,   // used by two handed fighting feature
	Finesse, // melee weapons use strength, ranged use dex, finesse take the better of either modifier
	Heavy,   // small or tiny get disadvantage on attack rolls when using this weapon
	Reach, // This weapon adds 5 feet to your reach when you attack with it, as well as when determining your reach for opportunity attacks with it.
	TwoHanded,
	Thrown(u32, u32),
	Versatile(Roll),
}

impl Property {
	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Light => "Light",
			Self::Finesse => "Finesse",
			Self::Heavy => "Heavy",
			Self::Reach => "Reach",
			Self::TwoHanded => "Two Handed",
			Self::Thrown(_, _) => "Thrown",
			Self::Versatile(_) => "Versatile",
		}
	}

	pub fn description(&self) -> String {
		match self {
			Self::Light => "When you use the Attack action to make a melee attack with this weapon, \
				you can use a bonus action to attack with a different light melee weapon \
				that you're holding in the other hand."
				.into(),
			Self::Finesse => "You can use either your Strength or Dexterity modifier \
				for both the attack and damage rolls."
				.into(),
			Self::Heavy => "Small or Tiny creatures have disadvantage on attack rolls with this weapon.".into(),
			Self::Reach => "This weapon extends an additional 5 feet of melee range when \
				making the attack action or opportunity attacks."
				.into(),
			Self::TwoHanded => "This weapon requires two hands when you attack with it.".into(),
			Self::Thrown(min, max) => format!(
				"You can throw this weapon to make a ranged attack, \
				with an inner-range of {min} and an outer-range of {max}."
			),
			Self::Versatile(roll) => format!(
				"This weapon can be used with one or two hands. \
				You deal {} damage when using two hands.",
				roll.to_string()
			),
		}
	}
}

impl FromKdl<NodeContext> for Property {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"Light" => Ok(Self::Light),
			"Finesse" => Ok(Self::Finesse),
			"Heavy" => Ok(Self::Heavy),
			"Reach" => Ok(Self::Reach),
			"TwoHanded" => Ok(Self::TwoHanded),
			"Thrown" => {
				let short = node.next_i64_req()? as u32;
				let long = node.next_i64_req()? as u32;
				Ok(Self::Thrown(short, long))
			}
			"Versatile" => {
				let roll = node.next_str_req_t::<Roll>()?;
				Ok(Self::Versatile(roll))
			}
			name => Err(GeneralError(format!("Unrecognized weapon property {name:?}")).into()),
		}
	}
}

impl AsKdl for Property {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Light => node.with_entry("Light"),
			Self::Finesse => node.with_entry("Finesse"),
			Self::Heavy => node.with_entry("Heavy"),
			Self::Reach => node.with_entry("Reach"),
			Self::TwoHanded => node.with_entry("TwoHanded"),
			Self::Thrown(short, long) => node
				.with_entry("Thrown")
				.with_entry(*short as i64)
				.with_entry(*long as i64),
			Self::Versatile(roll) => node.with_entry("Versatile").with_entry_typed(roll.to_string(), "Roll"),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::data::roll::Die};

		static NODE_NAME: &str = "property";

		#[test]
		fn light() -> anyhow::Result<()> {
			let doc = "property \"Light\"";
			let data = Property::Light;
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn finesse() -> anyhow::Result<()> {
			let doc = "property \"Finesse\"";
			let data = Property::Finesse;
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn heavy() -> anyhow::Result<()> {
			let doc = "property \"Heavy\"";
			let data = Property::Heavy;
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn reach() -> anyhow::Result<()> {
			let doc = "property \"Reach\"";
			let data = Property::Reach;
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn two_handed() -> anyhow::Result<()> {
			let doc = "property \"TwoHanded\"";
			let data = Property::TwoHanded;
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn thrown() -> anyhow::Result<()> {
			let doc = "property \"Thrown\" 20 60";
			let data = Property::Thrown(20, 60);
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn versatile() -> anyhow::Result<()> {
			let doc = "property \"Versatile\" (Roll)\"2d6\"";
			let data = Property::Versatile(Roll::from((2, Die::D6)));
			assert_eq_fromkdl!(Property, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
