use crate::kdl_ext::{DocumentExt, EntryExt, FromKDL, NodeContext, NodeExt, ValueExt};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Info {
	short: Option<String>,
	long: Vec<Section>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Section {
	pub title: Option<String>,
	pub content: String,
}

impl FromKDL for Info {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let short = node.query_str_opt("scope() > short", 0)?.map(str::to_owned);

		let mut long = Vec::new();
		long.push(Section::from_kdl(node, ctx)?);
		for node in node.query_all("scope() > section")? {
			long.push(Section::from_kdl(node, &mut ctx.next_node())?);
		}

		Ok(Self { short, long })
	}
}

impl FromKDL for Section {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let entry = node.entry_req(ctx.consume_idx())?;
		match entry.type_opt() {
			Some("Title") => {
				let title = Some(entry.as_str_req()?.to_owned());
				let content = node.get_str_req(ctx.consume_idx())?.to_owned();
				Ok(Self { title, content })
			}
			_ => {
				let content = entry.as_str_req()?.to_owned();
				Ok(Self {
					title: None,
					content,
				})
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::kdl_ext::NodeContext;

		fn from_doc(doc: &str) -> anyhow::Result<Info> {
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > description")?
				.expect("missing description node");
			Info::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn long_only() -> anyhow::Result<()> {
			let doc = "description \"This is some long description w/o a title\"";
			assert_eq!(
				from_doc(doc)?,
				Info {
					short: None,
					long: vec![Section {
						title: None,
						content: "This is some long description w/o a title".into()
					}]
				}
			);
			Ok(())
		}

		#[test]
		fn long_and_short() -> anyhow::Result<()> {
			let doc = "description \"This is some long description w/o a title\" {
				short \"Short desc of thing\"
			}";
			assert_eq!(
				from_doc(doc)?,
				Info {
					short: Some("Short desc of thing".into()),
					long: vec![Section {
						title: None,
						content: "This is some long description w/o a title".into()
					}]
				}
			);
			Ok(())
		}

		#[test]
		fn sections() -> anyhow::Result<()> {
			let doc = "description \"This is some long description w/o a title\" {
				short \"Short desc of thing\"
				section (Title)\"Title A\" \"desc for section A\"
				section \"desc for section B w/o title\"
				section (Title)\"Title C\" \"desc for section C\"
			}";
			assert_eq!(
				from_doc(doc)?,
				Info {
					short: Some("Short desc of thing".into()),
					long: vec![
						Section {
							title: None,
							content: "This is some long description w/o a title".into()
						},
						Section {
							title: Some("Title A".into()),
							content: "desc for section A".into()
						},
						Section {
							title: None,
							content: "desc for section B w/o title".into()
						},
						Section {
							title: Some("Title C".into()),
							content: "desc for section C".into()
						}
					]
				}
			);
			Ok(())
		}
	}
}
