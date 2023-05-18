use super::character::Character;
use crate::{
	kdl_ext::{DocumentExt, EntryExt, FromKDL, NodeContext, NodeExt, ValueExt},
	system::dnd5e::BoxedEvaluator,
	utility::SelectorMetaVec,
};
use std::{collections::HashMap, rc::Rc};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Info {
	pub short: Option<String>,
	pub sections: Vec<Section>,
	pub format_args: FormatArgs,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Section {
	pub title: Option<String>,
	pub content: SectionContent,
	pub format_args: FormatArgs,
	pub children: Vec<Section>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SectionContent {
	Body(String),
	Selectors(SelectorMetaVec),
	Table {
		column_count: usize,
		headers: Option<Vec<String>>,
		rows: Vec<Vec<String>>,
	},
}
impl Default for SectionContent {
	fn default() -> Self {
		Self::Body(String::default())
	}
}

impl Info {
	pub fn evaluate(mut self, state: &Character) -> Self {
		if !self.contains_format_syntax() {
			return self;
		}
		let args = self.format_args.evaluate(state);
		if let Some(short) = &mut self.short {
			FormatArgs::apply_to(short, &args);
		}
		for section in &mut self.sections {
			section.apply_args(state, &args);
		}
		self
	}

	fn contains_format_syntax(&self) -> bool {
		if let Some(short) = &self.short {
			if FormatArgs::contains_format_syntax(short.as_str()) {
				return true;
			}
		}
		for section in &self.sections {
			if section.contains_format_syntax() {
				return true;
			}
		}
		false
	}
}

impl FromKDL for Info {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let short = node.query_str_opt("scope() > short", 0)?.map(str::to_owned);

		let mut sections = Vec::new();
		for node in node.query_all("scope() > section")? {
			sections.push(Section::from_kdl(node, &mut ctx.next_node())?);
		}

		let format_args = FormatArgs::from_kdl_all(node, ctx)?;

		Ok(Self {
			short,
			sections,
			format_args,
		})
	}
}

impl Section {
	fn is_empty(&self) -> bool {
		self.content == SectionContent::default() && self.children.is_empty()
	}

	pub fn remove_selector_children(&mut self) {
		if matches!(self.content, SectionContent::Selectors(_)) {
			self.content = SectionContent::default();
		}

		let mut idx = 0;
		while idx < self.children.len() {
			self.children[idx].remove_selector_children();
			if self.children[idx].is_empty() {
				self.children.remove(idx);
			} else {
				idx += 1;
			}
		}
	}

	pub fn evaluate(mut self, state: &Character) -> Self {
		if !self.contains_format_syntax() {
			return self;
		}
		self.apply_args(state, &self.format_args.evaluate(state));
		self
	}

	fn contains_format_syntax(&self) -> bool {
		if self.content.contains_format_syntax() {
			return true;
		}
		for section in &self.children {
			if section.contains_format_syntax() {
				return true;
			}
		}
		false
	}

	fn apply_args(&mut self, state: &Character, parent_args: &HashMap<Rc<String>, Rc<String>>) {
		if !self.contains_format_syntax() {
			return;
		}

		let mut args = parent_args.clone();
		args.extend(self.format_args.evaluate(state));

		self.content.apply_args(&args);
		for section in &mut self.children {
			section.apply_args(state, &args);
		}
	}
}

impl From<SelectorMetaVec> for Section {
	fn from(value: SelectorMetaVec) -> Self {
		Self {
			content: SectionContent::Selectors(value),
			..Default::default()
		}
	}
}

impl FromKDL for Section {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		// Check if the first entry is a title.
		// There may not be a first entry (e.g. table) or the first entry might not be a title (title-less body),
		// so we cannot consume the first value in the node.
		let has_title = match node.entry_opt(ctx.peak_idx()) {
			Some(entry) => entry.type_opt() == Some("Title"),
			None => false,
		};
		// If the first value IS a title, read it and consume the entry.
		let title = match has_title {
			true => Some(node.get_str_req(ctx.consume_idx())?.to_owned()),
			false => None,
		};

		// Now parse the remaining values as the content.
		// This could be a body (using the next value) or a table (which checks properties and uses child nodes).
		let content = SectionContent::from_kdl(node, ctx)?;

		// Finally, read format args (if any exist) and any subsections/children.
		let format_args = FormatArgs::from_kdl_all(node, ctx)?;

		let mut children = Vec::new();
		for node in node.query_all("scope() > section")? {
			children.push(Section::from_kdl(node, &mut ctx.next_node())?);
		}

		Ok(Self {
			title,
			content,
			format_args,
			children,
		})
	}
}

impl SectionContent {
	fn contains_format_syntax(&self) -> bool {
		match self {
			Self::Body(content) => FormatArgs::contains_format_syntax(content.as_str()),
			_ => false,
		}
	}

	fn apply_args(&mut self, args: &HashMap<Rc<String>, Rc<String>>) {
		match self {
			Self::Body(content) => {
				FormatArgs::apply_to(content, args);
			}
			_ => {}
		}
	}
}

impl From<String> for SectionContent {
	fn from(value: String) -> Self {
		Self::Body(value)
	}
}

impl FromKDL for SectionContent {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let is_table = node.get_bool_opt("table")?.unwrap_or_default();
		match is_table {
			// Take note that we never return Self::Selectors.
			// That type is reserved specifically for hard/in-code descriptions (e.g. from mutators).
			false => {
				let content = node.get_str_req(ctx.consume_idx())?.to_owned();
				Ok(Self::Body(content))
			}
			true => {
				let headers = match node.query_opt("scope() > headers")? {
					None => None,
					Some(node) => {
						let mut columns = Vec::with_capacity(node.entries().len());
						for entry in node.entries() {
							columns.push(entry.as_str_req()?.to_owned());
						}
						Some(columns)
					}
				};
				let mut max_columns_in_rows = 0;
				let mut rows = Vec::new();
				for node in node.query_all("scope() > row")? {
					let col_count = node.entries().len();
					max_columns_in_rows = max_columns_in_rows.max(col_count);
					let mut columns = Vec::with_capacity(col_count);
					for entry in node.entries() {
						columns.push(entry.as_str_req()?.to_owned());
					}
					rows.push(columns);
				}
				let column_count = headers
					.as_ref()
					.map(|v| v.len())
					.unwrap_or(0)
					.max(max_columns_in_rows);
				Ok(Self::Table {
					column_count,
					headers,
					rows,
				})
			}
		}
	}
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct FormatArgs(HashMap<String, Arg>);

#[derive(Clone, PartialEq, Debug)]
struct Arg {
	evaluator: BoxedEvaluator<i32>,
	signed: bool,
}

impl<K, V> From<Vec<(K, V, bool)>> for FormatArgs
where
	K: Into<String>,
	V: Into<BoxedEvaluator<i32>>,
{
	fn from(values: Vec<(K, V, bool)>) -> Self {
		let mut map = HashMap::new();
		for (key, value, signed) in values {
			let arg = Arg {
				evaluator: value.into(),
				signed,
			};
			map.insert(key.into(), arg);
		}
		Self(map)
	}
}

impl FormatArgs {
	fn contains_format_syntax(text: &str) -> bool {
		lazy_static::lazy_static! {
			static ref RE: regex::Regex = regex::Regex::new("\\{.*?\\}").unwrap();
		}
		RE.is_match(text)
	}

	fn evaluate(&self, state: &Character) -> HashMap<Rc<String>, Rc<String>> {
		let mut evaluated = HashMap::default();
		for (key, arg) in &self.0 {
			let value = arg.evaluator.evaluate(state);
			let value = match arg.signed {
				true => format!("{value:+}"),
				false => format!("{value}"),
			};
			evaluated.insert(format!("{{{key}}}").into(), value.into());
		}
		evaluated
	}

	fn apply_to(target: &mut String, args: &HashMap<Rc<String>, Rc<String>>) {
		for (key, value) in args {
			if target.contains(key.as_str()) {
				*target = target.replace(key.as_str(), value.as_str());
			}
		}
	}

	/// Queries `node` for all child nodes with the name `format-arg`,
	/// parsing each as a named evaluator argument for the list.
	pub fn from_kdl_all(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let mut args = HashMap::new();
		for node in node.query_all("scope() > format-arg")? {
			let mut ctx = ctx.next_node();
			let key_entry = node.entry_req(ctx.consume_idx())?;
			let key = key_entry.as_str_req()?.to_owned();
			let signed = key_entry.type_opt() == Some("Signed");
			let evaluator = ctx.parse_evaluator_inline(node)?;
			let arg = Arg { evaluator, signed };
			args.insert(key, arg);
		}
		Ok(Self(args))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{
				core::NodeRegistry,
				dnd5e::{data::Ability, evaluator::GetAbilityModifier},
			},
		};

		fn node_ctx() -> NodeContext {
			let mut reg = NodeRegistry::default();
			reg.register_evaluator::<GetAbilityModifier>();
			NodeContext::registry(reg)
		}

		mod info {
			use super::*;

			fn from_doc(doc: &str) -> anyhow::Result<Info> {
				let document = doc.parse::<kdl::KdlDocument>()?;
				let node = document
					.query("scope() > description")?
					.expect("missing description node");
				Info::from_kdl(node, &mut node_ctx())
			}

			#[test]
			fn long_only() -> anyhow::Result<()> {
				let doc = "description {
					section \"This is some long description w/o a title\"
				}";
				assert_eq!(
					from_doc(doc)?,
					Info {
						short: None,
						sections: vec![Section {
							title: None,
							content: SectionContent::Body(
								"This is some long description w/o a title".into()
							),
							format_args: FormatArgs::default(),
							children: vec![],
						}],
						format_args: FormatArgs::default(),
					}
				);
				Ok(())
			}

			#[test]
			fn short_only() -> anyhow::Result<()> {
				let doc = "description {
					short \"This is some short description\"
				}";
				assert_eq!(
					from_doc(doc)?,
					Info {
						short: Some("This is some short description".into()),
						sections: vec![],
						format_args: FormatArgs::default(),
					}
				);
				Ok(())
			}

			#[test]
			fn format_args() -> anyhow::Result<()> {
				let doc = "description {
					short \"Success against a DC {DC} Wisdom saving throw\"
					format-arg \"DC\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
				}";
				assert_eq!(
					from_doc(doc)?,
					Info {
						short: Some("Success against a DC {DC} Wisdom saving throw".into()),
						sections: vec![],
						format_args: FormatArgs::from(vec![(
							"DC",
							GetAbilityModifier(Ability::Intelligence),
							false,
						)]),
					}
				);
				Ok(())
			}
		}

		mod section {
			use super::*;

			fn from_doc(doc: &str) -> anyhow::Result<Section> {
				let document = doc.parse::<kdl::KdlDocument>()?;
				let node = document
					.query("scope() > section")?
					.expect("missing section node");
				Section::from_kdl(node, &mut node_ctx())
			}

			#[test]
			fn body_only() -> anyhow::Result<()> {
				let doc = "section \"Simple body-only section\"";
				assert_eq!(
					from_doc(doc)?,
					Section {
						title: None,
						content: SectionContent::Body("Simple body-only section".into()),
						format_args: FormatArgs::default(),
						children: vec![],
					}
				);
				Ok(())
			}

			#[test]
			fn titled_body() -> anyhow::Result<()> {
				let doc = "section (Title)\"Section A\" \"Body with a title!\"";
				assert_eq!(
					from_doc(doc)?,
					Section {
						title: Some("Section A".into()),
						content: SectionContent::Body("Body with a title!".into()),
						format_args: FormatArgs::default(),
						children: vec![],
					}
				);
				Ok(())
			}

			#[test]
			fn table() -> anyhow::Result<()> {
				let doc = "section table=true {
					headers \"Col 1\" \"Col 2\"	\"Col 3\"
					row \"R1 C1\" \"R1 C2\" \"R1 C3\"
					row \"R2 C1\" \"R2 C2\" \"R2 C3\"
				}";
				assert_eq!(
					from_doc(doc)?,
					Section {
						title: None,
						content: SectionContent::Table {
							column_count: 3,
							headers: Some(vec!["Col 1".into(), "Col 2".into(), "Col 3".into()]),
							rows: vec![
								vec!["R1 C1".into(), "R1 C2".into(), "R1 C3".into()],
								vec!["R2 C1".into(), "R2 C2".into(), "R2 C3".into()],
							]
						},
						format_args: FormatArgs::default(),
						children: vec![],
					}
				);
				Ok(())
			}

			#[test]
			fn format_args() -> anyhow::Result<()> {
				let doc = "section \"Body with {num} format-args\" {
					format-arg \"num\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
				}";
				assert_eq!(
					from_doc(doc)?,
					Section {
						title: None,
						content: SectionContent::Body("Body with {num} format-args".into()),
						format_args: FormatArgs::from(vec![(
							"num",
							GetAbilityModifier(Ability::Intelligence),
							false,
						)]),
						children: vec![],
					}
				);
				Ok(())
			}

			#[test]
			fn children() -> anyhow::Result<()> {
				let doc = "section \"main body\" {
					section (Title)\"Subsection A\" \"subsection A body\"
					section \"subsection B body\"
				}";
				assert_eq!(
					from_doc(doc)?,
					Section {
						title: None,
						content: SectionContent::Body("main body".into()),
						format_args: FormatArgs::default(),
						children: vec![
							Section {
								title: Some("Subsection A".into()),
								content: SectionContent::Body("subsection A body".into()),
								format_args: FormatArgs::default(),
								children: vec![],
							},
							Section {
								title: None,
								content: SectionContent::Body("subsection B body".into()),
								format_args: FormatArgs::default(),
								children: vec![],
							},
						],
					}
				);
				Ok(())
			}
		}

		mod format_args {
			use super::*;

			fn from_doc(doc: &str) -> anyhow::Result<FormatArgs> {
				let document = doc.parse::<kdl::KdlDocument>()?;
				let node = document
					.query("scope() > args")?
					.expect("missing args node");
				FormatArgs::from_kdl_all(node, &mut node_ctx())
			}

			#[test]
			fn unsigned() -> anyhow::Result<()> {
				let doc = "args {
					format-arg \"DC\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
				}";
				assert_eq!(
					from_doc(doc)?,
					FormatArgs::from(vec![(
						"DC",
						GetAbilityModifier(Ability::Intelligence),
						false,
					)]),
				);
				Ok(())
			}

			#[test]
			fn signed() -> anyhow::Result<()> {
				let doc = "args {
					format-arg (Signed)\"DC\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
				}";
				assert_eq!(
					from_doc(doc)?,
					FormatArgs::from(vec![(
						"DC",
						GetAbilityModifier(Ability::Intelligence),
						true,
					)]),
				);
				Ok(())
			}
		}
	}

	mod evaluate {
		use super::*;
		use crate::system::dnd5e::{
			data::{character::Persistent, Ability},
			evaluator::GetAbilityModifier,
		};

		fn character(scores: Vec<(Ability, u32)>) -> Character {
			let mut persistent = Persistent::default();
			for (ability, score) in scores {
				persistent.ability_scores[ability] = score;
			}
			Character::from(persistent)
		}

		#[test]
		fn simple() {
			let character = character(vec![
				(Ability::Strength, 12),
				(Ability::Wisdom, 14),
				(Ability::Intelligence, 20),
			]);
			let info = Info {
				short: Some("{DC} Wis Save or take {num} force damage".into()),
				sections: vec![Section {
					title: None,
					content: SectionContent::Body(
						"In addition to {num} dmg on failed {DC} save, take {fire} fire damage."
							.into(),
					),
					children: vec![],
					format_args: FormatArgs::from(vec![(
						"fire",
						GetAbilityModifier(Ability::Strength),
						true,
					)]),
				}],
				format_args: FormatArgs::from(vec![
					("DC", GetAbilityModifier(Ability::Intelligence), false),
					("num", GetAbilityModifier(Ability::Wisdom), true),
				]),
			};
			let expected = Info {
				short: Some("5 Wis Save or take +2 force damage".into()),
				sections: vec![Section {
					title: None,
					content: SectionContent::Body(
						"In addition to +2 dmg on failed 5 save, take +1 fire damage.".into(),
					),
					children: vec![],
					format_args: FormatArgs::from(vec![(
						"fire",
						GetAbilityModifier(Ability::Strength),
						true,
					)]),
				}],
				format_args: FormatArgs::from(vec![
					("DC", GetAbilityModifier(Ability::Intelligence), false),
					("num", GetAbilityModifier(Ability::Wisdom), true),
				]),
			};
			assert_eq!(info.evaluate(&character), expected);
		}
	}
}
