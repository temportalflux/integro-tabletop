use crate::{
	kdl_ext::{FromKDL, NodeContext, NodeExt, ValueExt},
	utility::NotInList,
};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum Range {
	#[default]
	OnlySelf,
	Touch,
	Unit {
		distance: u64,
		unit: String,
	},
	Sight,
	Unlimited,
}

impl FromKDL for Range {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let entry = node.entry_req(ctx.consume_idx())?;
		match entry.value().as_string() {
			None => {
				let distance = entry.as_i64_req()? as u64;
				let unit = node
					.get_str_opt(ctx.consume_idx())?
					.unwrap_or("Feet")
					.to_owned();
				Ok(Self::Unit { distance, unit })
			}
			Some("Self") => Ok(Self::OnlySelf),
			Some("Touch") => Ok(Self::Touch),
			Some("Sight") => Ok(Self::Sight),
			Some("Unlimited") => Ok(Self::Unlimited),
			Some(type_name) => Err(NotInList(
				type_name.into(),
				vec!["Self", "Touch", "Sight", "Unlimited"],
			)
			.into()),
		}
	}
}
