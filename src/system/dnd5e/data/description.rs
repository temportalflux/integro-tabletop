use crate::{
	kdl_ext::{DocumentExt, EntryExt, FromKDL, NodeContext, NodeExt, ValueExt},
	utility::SelectorMetaVec,
};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Info {
	pub short: Option<String>,
	pub long: Vec<Section>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Section {
	pub title: Option<String>,
	pub content: String,
	pub selectors: SelectorMetaVec,
	pub kind: Option<SectionKind>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SectionKind {
	HasChildren(Vec<Section>),
	//Table,
}

impl<S> From<S> for Info
where
	S: Into<String>,
{
	fn from(value: S) -> Self {
		Self {
			short: None,
			long: vec![Section {
				content: value.into(),
				..Default::default()
			}],
		}
	}
}

impl Info {
	/// Queries the children of `parent` for any nodes named `description`,
	/// and extends the default `Info` with all identified children parsed as `Info`.
	pub fn from_kdl_all(parent: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Info> {
		let mut info = Info::default();
		for node in parent.query_all("scope() > description")? {
			info.extend(Info::from_kdl(node, &mut ctx.next_node())?);
		}
		Ok(info)
	}

	pub fn extend(&mut self, other: Self) {
		if self.short.is_none() && other.short.is_some() {
			self.short = other.short();
		}
		self.long.extend(other.long);
	}

	pub fn short(&self) -> Option<String> {
		self.short.as_ref().map(|s| self.format_desc(s))
	}

	pub fn long(&self) -> impl Iterator<Item = Section> + '_ {
		self.long.iter().map(|section| Section {
			title: section.title.clone(),
			content: self.format_desc(&section.content),
			..Default::default()
		})
	}

	fn format_desc(&self, text: &String) -> String {
		text.clone()
	}
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
				Ok(Self {
					title,
					content,
					..Default::default()
				})
			}
			_ => {
				let content = entry.as_str_req()?.to_owned();
				Ok(Self {
					content,
					..Default::default()
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
						content: "This is some long description w/o a title".into(),
						..Default::default()
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
						content: "This is some long description w/o a title".into(),
						..Default::default()
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
							content: "This is some long description w/o a title".into(),
							..Default::default()
						},
						Section {
							title: Some("Title A".into()),
							content: "desc for section A".into(),
							..Default::default()
						},
						Section {
							content: "desc for section B w/o title".into(),
							..Default::default()
						},
						Section {
							title: Some("Title C".into()),
							content: "desc for section C".into(),
							..Default::default()
						}
					]
				}
			);
			Ok(())
		}
	}
}
