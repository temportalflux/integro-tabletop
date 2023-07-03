use super::character::Character;
use crate::{
	kdl_ext::{AsKdl, DocumentExt, EntryExt, FromKDL, NodeBuilder, ValueExt},
	system::dnd5e::BoxedEvaluator,
	utility::{NotInList, SelectorMetaVec},
};
use std::{
	collections::{BTreeMap, HashMap},
	rc::Rc,
};

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
	pub fn is_empty(&self) -> bool {
		self.sections.is_empty()
	}

	pub fn evaluate(self, state: &Character) -> Self {
		self.evaluate_with(state, None)
	}

	pub fn evaluate_with(
		mut self,
		state: &Character,
		args: Option<HashMap<String, String>>,
	) -> Self {
		if !self.contains_format_syntax() {
			return self;
		}
		let mut all_args = self.format_args.evaluate(state);
		if let Some(parent_args) = args {
			for (key, value) in parent_args {
				all_args.insert(key.into(), value.into());
			}
		}
		if let Some(short) = &mut self.short {
			FormatArgs::apply_to(short, &all_args);
		}
		for section in &mut self.sections {
			section.apply_args(state, &all_args);
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
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		// If there are no children, this can be treated as a single-section block,
		// where the section is the long description.
		if node.children().is_none() {
			let section = Section::from_kdl(node)?;
			return Ok(Self {
				short: None,
				sections: vec![section],
				format_args: FormatArgs::default(),
			});
		}

		let short = node.query_str_opt("scope() > short", 0)?.map(str::to_owned);
		let sections = node.query_all_t::<Section>("scope() > section")?;
		let format_args = FormatArgs::from_kdl_all(&node)?;

		Ok(Self {
			short,
			sections,
			format_args,
		})
	}
}
impl AsKdl for Info {
	fn as_kdl(&self) -> NodeBuilder {
		if self.short.is_none() && self.format_args.0.is_empty() && self.sections.len() == 1 {
			return self.sections[0].as_kdl();
		}

		let mut node = NodeBuilder::default();

		if let Some(short) = &self.short {
			node.push_child_t("short", short);
		}

		for section in &self.sections {
			node.push_child_t("section", section);
		}

		node += self.format_args.as_kdl();

		node
	}
}

impl Section {
	pub fn is_empty(&self) -> bool {
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
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		// Check if the first entry is a title.
		// There may not be a first entry (e.g. table) or the first entry might not be a title (title-less body),
		// so we cannot consume the first value in the node.
		let has_title = match node.peak_opt() {
			Some(entry) => entry.type_opt() == Some("Title"),
			None => false,
		};
		// If the first value IS a title, read it and consume the entry.
		let title = match has_title {
			true => Some(node.next_str_req()?.to_owned()),
			false => None,
		};

		// Now parse the remaining values as the content.
		// This could be a body (using the next value) or a table (which checks properties and uses child nodes).
		let content = SectionContent::from_kdl(node)?;

		// Finally, read format args (if any exist) and any subsections/children.
		let format_args = FormatArgs::from_kdl_all(&node)?;

		let mut children = Vec::new();
		for mut node in &mut node.query_all("scope() > section")? {
			children.push(Section::from_kdl(&mut node)?);
		}

		Ok(Self {
			title,
			content,
			format_args,
			children,
		})
	}
}
impl AsKdl for Section {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if let Some(title) = &self.title {
			let mut entry = kdl::KdlEntry::new(title.clone());
			entry.set_ty("Title");
			node.push_entry(entry);
		}

		node += self.content.as_kdl();

		for section in &self.children {
			node.push_child_opt_t("section", section);
		}

		node += self.format_args.as_kdl();

		node
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
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let is_table = node.get_bool_opt("table")?.unwrap_or_default();
		match is_table {
			// Take note that we never return Self::Selectors.
			// That type is reserved specifically for hard/in-code descriptions (e.g. from mutators).
			false => {
				let content = node.next_str_req()?.to_owned();
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
				for node in &mut node.query_all("scope() > row")? {
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
impl AsKdl for SectionContent {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Body(content) => {
				node.push_entry(content.clone());
			}
			Self::Table {
				column_count: _,
				headers,
				rows,
			} => {
				node.push_entry(("table", true));
				if let Some(headers) = headers {
					node.push_child({
						let mut node = NodeBuilder::default();
						for name in headers {
							node.push_entry(name.clone());
						}
						node.build("headers")
					});
				}
				for row in rows {
					let mut row_node = NodeBuilder::default();
					for col in row {
						row_node.push_entry(col.clone());
					}
					node.push_child(row_node.build("row"));
				}
			}
			Self::Selectors(_) => {}
		}
		node
	}
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct FormatArgs(BTreeMap<String, Arg>);

#[derive(Clone, PartialEq, Debug)]
enum Arg {
	Number(BoxedEvaluator<i32>, bool),
	String(BoxedEvaluator<String>),
}

impl<K, V> From<Vec<(K, V, bool)>> for FormatArgs
where
	K: Into<String>,
	V: Into<BoxedEvaluator<i32>>,
{
	fn from(values: Vec<(K, V, bool)>) -> Self {
		let mut map = BTreeMap::new();
		for (key, value, signed) in values {
			let arg = Arg::Number(value.into(), signed);
			map.insert(key.into(), arg);
		}
		Self(map)
	}
}
impl<K, V> From<Vec<(K, V)>> for FormatArgs
where
	K: Into<String>,
	V: Into<BoxedEvaluator<String>>,
{
	fn from(values: Vec<(K, V)>) -> Self {
		let mut map = BTreeMap::new();
		for (key, value) in values {
			let arg = Arg::String(value.into());
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
			let value = match arg {
				Arg::Number(eval, signed) => {
					let value = eval.evaluate(state);
					match *signed {
						true => format!("{value:+}"),
						false => format!("{value}"),
					}
				}
				Arg::String(eval) => eval.evaluate(state),
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
	pub fn from_kdl_all<'doc>(node: &crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut args = BTreeMap::new();
		for node in &mut node.query_all("scope() > format-arg")? {
			let key = node.next_str_req()?.to_owned();
			let eval_type_entry = node.next_req()?;
			let arg = match eval_type_entry.as_str_req()? {
				"int" => {
					let signed = eval_type_entry.type_opt() == Some("Signed");
					Arg::Number(node.parse_evaluator_inline()?, signed)
				}
				"str" => Arg::String(node.parse_evaluator_inline()?),
				_type => return Err(NotInList(_type.into(), vec!["int", "str"]).into()),
			};
			args.insert(key, arg);
		}
		Ok(Self(args))
	}
}
impl AsKdl for FormatArgs {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		for (key, arg) in &self.0 {
			let mut arg_node = NodeBuilder::default().with_entry(key.clone());
			let eval_kdl = match arg {
				Arg::Number(eval, signed) => {
					let mut entry = kdl::KdlEntry::new("int");
					if *signed {
						entry.set_ty("Signed");
					}
					arg_node.push_entry(entry);
					eval.as_kdl()
				}
				Arg::String(eval) => {
					arg_node.push_entry("str");
					eval.as_kdl()
				}
			};
			arg_node.append_typed("Evaluator", eval_kdl);
			node.push_child(arg_node.build("format-arg"));
		}

		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{
				core::NodeRegistry,
				dnd5e::{data::Ability, evaluator::GetAbilityModifier},
			},
		};

		fn node_ctx() -> NodeContext {
			NodeContext::registry(NodeRegistry::default_with_eval::<GetAbilityModifier>())
		}

		mod info {
			use super::*;

			static NODE_NAME: &str = "description";

			#[test]
			fn long_only_simple() -> anyhow::Result<()> {
				let doc = "description \"This is some long description w/o a title\"";
				let data = Info {
					short: None,
					sections: vec![Section {
						title: None,
						content: SectionContent::Body(
							"This is some long description w/o a title".into(),
						),
						format_args: FormatArgs::default(),
						children: vec![],
					}],
					format_args: FormatArgs::default(),
				};
				assert_eq_fromkdl!(Info, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn long_only() -> anyhow::Result<()> {
				let doc_in = "
					|description {
					|    section \"This is some long description w/o a title\"
					|}
				";
				let doc_out = "description \"This is some long description w/o a title\"";
				let data = Info {
					short: None,
					sections: vec![Section {
						title: None,
						content: SectionContent::Body(
							"This is some long description w/o a title".into(),
						),
						format_args: FormatArgs::default(),
						children: vec![],
					}],
					format_args: FormatArgs::default(),
				};
				assert_eq_fromkdl!(Info, doc_in, data);
				assert_eq_askdl!(&data, doc_out);
				Ok(())
			}

			#[test]
			fn short_only() -> anyhow::Result<()> {
				let doc = "
					|description {
					|    short \"This is some short description\"
					|}
				";
				let data = Info {
					short: Some("This is some short description".into()),
					sections: vec![],
					format_args: FormatArgs::default(),
				};
				assert_eq_fromkdl!(Info, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn format_args() -> anyhow::Result<()> {
				let doc = "
					|description {
					|    short \"Success against a DC {DC} Wisdom saving throw\"
					|    format-arg \"DC\" \"int\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
					|}
				";
				let data = Info {
					short: Some("Success against a DC {DC} Wisdom saving throw".into()),
					sections: vec![],
					format_args: FormatArgs::from(vec![(
						"DC",
						GetAbilityModifier(Ability::Intelligence),
						false,
					)]),
				};
				assert_eq_fromkdl!(Info, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}
		}

		mod section {
			use super::*;

			static NODE_NAME: &str = "section";

			#[test]
			fn body_only() -> anyhow::Result<()> {
				let doc = "section \"Simple body-only section\"";
				let data = Section {
					title: None,
					content: SectionContent::Body("Simple body-only section".into()),
					format_args: FormatArgs::default(),
					children: vec![],
				};
				assert_eq_fromkdl!(Section, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn titled_body() -> anyhow::Result<()> {
				let doc = "section (Title)\"Section A\" \"Body with a title!\"";
				let data = Section {
					title: Some("Section A".into()),
					content: SectionContent::Body("Body with a title!".into()),
					format_args: FormatArgs::default(),
					children: vec![],
				};
				assert_eq_fromkdl!(Section, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn table() -> anyhow::Result<()> {
				let doc = "
					|section table=true {
					|    headers \"Col 1\" \"Col 2\" \"Col 3\"
					|    row \"R1 C1\" \"R1 C2\" \"R1 C3\"
					|    row \"R2 C1\" \"R2 C2\" \"R2 C3\"
					|}
				";
				let data = Section {
					title: None,
					content: SectionContent::Table {
						column_count: 3,
						headers: Some(vec!["Col 1".into(), "Col 2".into(), "Col 3".into()]),
						rows: vec![
							vec!["R1 C1".into(), "R1 C2".into(), "R1 C3".into()],
							vec!["R2 C1".into(), "R2 C2".into(), "R2 C3".into()],
						],
					},
					format_args: FormatArgs::default(),
					children: vec![],
				};
				assert_eq_fromkdl!(Section, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn format_args() -> anyhow::Result<()> {
				let doc = "
					|section \"Body with {num} format-args\" {
					|    format-arg \"num\" \"int\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
					|}
				";
				let data = Section {
					title: None,
					content: SectionContent::Body("Body with {num} format-args".into()),
					format_args: FormatArgs::from(vec![(
						"num",
						GetAbilityModifier(Ability::Intelligence),
						false,
					)]),
					children: vec![],
				};
				assert_eq_fromkdl!(Section, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn children() -> anyhow::Result<()> {
				let doc = "
					|section \"main body\" {
					|    section (Title)\"Subsection A\" \"subsection A body\"
					|    section \"subsection B body\"
					|}
				";
				let data = Section {
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
				};
				assert_eq_fromkdl!(Section, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}
		}

		mod format_args {
			use super::*;

			static NODE_NAME: &str = "args";

			fn from_kdl<'doc>(
				node: crate::kdl_ext::NodeReader<'doc>,
			) -> anyhow::Result<FormatArgs> {
				FormatArgs::from_kdl_all(&node)
			}

			#[test]
			fn unsigned() -> anyhow::Result<()> {
				let doc = "
					|args {
					|    format-arg \"DC\" \"int\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
					|}
				";
				let data = FormatArgs::from(vec![(
					"DC",
					GetAbilityModifier(Ability::Intelligence),
					false,
				)]);
				assert_eq_fromkdl!(FormatArgs, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn signed() -> anyhow::Result<()> {
				let doc = "
					|args {
					|    format-arg \"DC\" (Signed)\"int\" (Evaluator)\"get_ability_modifier\" (Ability)\"Intelligence\"
					|}
				";
				let data = FormatArgs::from(vec![(
					"DC",
					GetAbilityModifier(Ability::Intelligence),
					true,
				)]);
				assert_eq_fromkdl!(FormatArgs, doc, data);
				assert_eq_askdl!(&data, doc);
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
