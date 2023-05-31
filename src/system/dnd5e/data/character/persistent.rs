use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeContext, NodeExt},
	path_map::PathMap,
	system::{
		core::SourceId,
		dnd5e::{
			data::{
				bundle::{Background, Lineage, Race, RaceVariant, Upbringing},
				character::Character,
				item, Ability, Class, Condition, Feature, Spell,
			},
			SystemComponent,
		},
	},
	utility::{MutatorGroup, NotInList},
};
use enum_map::EnumMap;
use std::{
	collections::{BTreeMap, HashMap},
	path::Path,
	str::FromStr,
	sync::Arc,
};

mod description;
pub use description::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct NamedGroups {
	pub race: Vec<Race>,
	pub race_variant: Vec<RaceVariant>,
	pub lineage: Vec<Lineage>,
	pub upbringing: Vec<Upbringing>,
	pub background: Vec<Background>,
}

/// Core character data which is (de)serializable and
/// from which the derived data can be compiled.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Persistent {
	pub named_groups: NamedGroups,
	pub classes: Vec<Class>,
	pub feats: Vec<Feature>,
	pub description: Description,
	pub ability_scores: EnumMap<Ability, u32>,
	pub selected_values: PathMap<String>,
	pub selected_spells: SelectedSpells,
	pub inventory: item::Inventory<item::EquipableEntry>,
	pub conditions: Conditions,
	pub hit_points: HitPoints,
	pub inspiration: bool,
	pub settings: Settings,
}
impl MutatorGroup for Persistent {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		for group in &self.named_groups.race {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.race_variant {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.lineage {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.upbringing {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.background {
			group.set_data_path(parent);
		}
		for group in &self.classes {
			group.set_data_path(parent);
		}
		for group in &self.feats {
			group.set_data_path(parent);
		}
		self.inventory.set_data_path(parent);
		self.conditions.set_data_path(parent);
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		for (ability, score) in &self.ability_scores {
			stats
				.ability_scores_mut()
				.push_bonus(ability, (*score).into(), "Base Score".into());
		}
		stats.apply(&super::FinalizeAbilityScores.into(), parent);

		for group in &self.named_groups.race {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.race_variant {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.lineage {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.upbringing {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.background {
			stats.apply_from(group, parent);
		}
		for class in &self.classes {
			stats.apply_from(class, parent);
		}
		for feat in &self.feats {
			stats.add_feature(feat, parent);
		}
		stats.apply_from(&self.conditions, parent);
		stats.apply_from(&self.inventory, parent);
	}
}

impl Persistent {
	pub fn add_class(&mut self, mut class: Class) {
		class.levels.truncate(1);
		self.classes.push(class);
	}

	pub fn level(&self, class_name: Option<&str>) -> usize {
		match class_name {
			Some(class_name) => {
				for class in &self.classes {
					if class.name == class_name {
						return class.current_level;
					}
				}
				return 0;
			}
			None => self.classes.iter().map(|class| class.current_level).sum(),
		}
	}

	pub fn hit_points(&self) -> &HitPoints {
		&self.hit_points
	}

	pub fn hit_points_mut(&mut self) -> &mut HitPoints {
		&mut self.hit_points
	}

	pub fn set_selected_value(&mut self, key: impl AsRef<Path>, value: impl Into<String>) {
		self.selected_values.set(key, value.into());
	}

	pub fn set_selected(&mut self, key: impl AsRef<Path>, value: Option<String>) {
		match value {
			Some(value) => {
				self.selected_values.set(key, value);
			}
			None => {
				let _ = self.selected_values.remove(key);
			}
		}
	}

	pub fn insert_selection(&mut self, key: impl AsRef<Path>, value: impl Into<String>) {
		self.selected_values.insert(key, value.into());
	}

	pub fn remove_selection(&mut self, key: impl AsRef<Path>, index: usize) -> Option<String> {
		let Some(values) = self.selected_values.get_mut(key) else { return None; };
		if index < values.len() {
			Some(values.remove(index))
		} else {
			None
		}
	}
}

crate::impl_kdl_node!(Persistent, "character");
impl SystemComponent for Persistent {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": &self.description.name,
		})
	}
}
impl FromKDL for Persistent {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		ctx.set_inheiret_source(false);

		let description = Description::from_kdl(
			node.query_req("scope() > description")?,
			&mut ctx.next_node(),
		)?;

		let mut settings = Settings::default();
		for node in node.query_all("scope() > setting")? {
			settings.insert_from_kdl(node, &mut ctx.next_node())?;
		}

		let mut ability_scores = EnumMap::default();
		for node in node.query_all("scope() > ability")? {
			let mut ctx = ctx.next_node();
			let ability = Ability::from_str(node.get_str_req(ctx.consume_idx())?)?;
			let score = node.get_i64_req(ctx.consume_idx())? as u32;
			ability_scores[ability] = score;
		}

		let hit_points = HitPoints::from_kdl(
			node.query_req("scope() > hit_points")?,
			&mut ctx.next_node(),
		)?;

		let inspiration = node
			.query_bool_opt("scope() > inspiration", 0)?
			.unwrap_or_default();

		let mut conditions = Conditions::default();
		for node in node.query_all("scope() > condition")? {
			let condition = Condition::from_kdl(node, &mut ctx.next_node())?;
			conditions.insert(condition);
		}

		let inventory = item::Inventory::from_kdl(
			node.query_req("scope() > inventory")?,
			&mut ctx.next_node(),
		)?;

		let selected_spells = match node.query_opt("scope() > spells")? {
			Some(node) => SelectedSpells::from_kdl(node, &mut ctx.next_node())?,
			None => SelectedSpells::default(),
		};

		// TODO: Technically all named groups are also features, just with a different category.
		let mut named_groups = NamedGroups::default();
		let mut feats = Vec::new();
		for node in node.query_all("scope() > feat")? {
			let mut ctx = ctx.next_node();
			match node.get_str_req("category")? {
				"Feat" => {
					let feature = Feature::from_kdl(node, &mut ctx)?;
					feats.push(feature);
				}
				"Race" => {
					let feature = Race::from_kdl(node, &mut ctx)?;
					named_groups.race.push(feature);
				}
				"Subrace" => {
					let feature = RaceVariant::from_kdl(node, &mut ctx)?;
					named_groups.race_variant.push(feature);
				}
				"Lineage" => {
					let feature = Lineage::from_kdl(node, &mut ctx)?;
					named_groups.lineage.push(feature);
				}
				"Upbringing" => {
					let feature = Upbringing::from_kdl(node, &mut ctx)?;
					named_groups.upbringing.push(feature);
				}
				"Background" => {
					let feature = Background::from_kdl(node, &mut ctx)?;
					named_groups.background.push(feature);
				}
				category => {
					return Err(NotInList(
						category.into(),
						vec![
							"Race",
							"Subrace",
							"Lineage",
							"Upbringing",
							"Background",
							"Feat",
						],
					)
					.into());
				}
			}
		}

		let mut classes = Vec::new();
		for node in node.query_all("scope() > class")? {
			let class = Class::from_kdl(node, &mut ctx.next_node())?;
			classes.push(class);
		}

		let mut selected_values = PathMap::<String>::default();
		if let Some(selections) = node.query_opt("scope() > selections")? {
			for node in selections.query_all("scope() > value")? {
				let mut ctx = ctx.next_node();
				let key_str = node.get_str_req(ctx.consume_idx())?;
				let value = node.get_str_req(ctx.consume_idx())?.to_owned();
				selected_values.insert(Path::new(key_str), value);
			}
		}

		Ok(Self {
			description,
			settings,
			ability_scores,
			hit_points,
			inspiration,
			conditions,
			inventory,
			selected_spells,
			named_groups,
			feats,
			classes,
			selected_values,
		})
	}
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct HitPoints {
	pub current: u32,
	pub temp: u32,
	pub failure_saves: u8,
	pub success_saves: u8,
}
impl FromKDL for HitPoints {
	fn from_kdl(node: &kdl::KdlNode, _ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let current = node.query_i64_req("scope() > current", 0)? as u32;
		let temp = node.query_i64_req("scope() > temp", 0)? as u32;
		let failure_saves = node.query_i64_req("scope() > failure_saves", 0)? as u8;
		let success_saves = node.query_i64_req("scope() > success_saves", 0)? as u8;
		Ok(Self {
			current,
			temp,
			failure_saves,
			success_saves,
		})
	}
}
impl HitPoints {
	pub fn set_temp_hp(&mut self, value: u32) {
		self.temp = value;
	}

	pub fn plus_hp(mut self, amount: i32, max: u32) -> Self {
		let mut amt_abs = amount.abs() as u32;
		let had_hp = self.current > 0;
		match amount.signum() {
			1 => {
				self.current = self.current.saturating_add(amt_abs).min(max);
			}
			-1 if self.temp >= amt_abs => {
				self.temp = self.temp.saturating_sub(amt_abs);
			}
			-1 if self.temp < amt_abs => {
				amt_abs -= self.temp;
				self.temp = 0;
				self.current = self.current.saturating_sub(amt_abs);
			}
			_ => {}
		}
		if !had_hp && self.current != 0 {
			self.failure_saves = 0;
			self.success_saves = 0;
		}
		self
	}
}
impl std::ops::Add<(i32, u32)> for HitPoints {
	type Output = Self;

	fn add(self, (amount, max): (i32, u32)) -> Self::Output {
		self.plus_hp(amount, max)
	}
}
impl std::ops::AddAssign<(i32, u32)> for HitPoints {
	fn add_assign(&mut self, rhs: (i32, u32)) {
		*self = *self + rhs;
	}
}

#[derive(Clone)]
pub enum IdOrIndex {
	Id(Arc<SourceId>),
	Index(usize),
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Conditions {
	by_id: BTreeMap<SourceId, Condition>,
	custom: Vec<Condition>,
}
impl Conditions {
	pub fn insert(&mut self, condition: Condition) {
		match &condition.id {
			Some(id) => {
				self.by_id.insert(id.unversioned(), condition);
			}
			None => {
				self.custom.push(condition);
				self.custom.sort_by(|a, b| a.name.cmp(&b.name));
			}
		}
	}

	pub fn remove(&mut self, key: &IdOrIndex) {
		match key {
			IdOrIndex::Id(id) => {
				self.by_id.remove(&*id);
			}
			IdOrIndex::Index(idx) => {
				self.custom.remove(*idx);
			}
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &Condition> {
		self.by_id.values().chain(self.custom.iter())
	}

	pub fn iter_keyed(&self) -> impl Iterator<Item = (IdOrIndex, &Condition)> {
		let ids = self
			.by_id
			.iter()
			.map(|(id, value)| (IdOrIndex::Id(Arc::new(id.clone())), value));
		let indices = self
			.custom
			.iter()
			.enumerate()
			.map(|(idx, value)| (IdOrIndex::Index(idx), value));
		ids.chain(indices)
	}

	pub fn contains_id(&self, id: &SourceId) -> bool {
		self.by_id.contains_key(id)
	}
}
impl MutatorGroup for Conditions {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		for condition in self.iter() {
			condition.set_data_path(parent);
		}
	}

	fn apply_mutators(&self, target: &mut Self::Target, parent: &Path) {
		for condition in self.iter() {
			target.apply_from(condition, parent);
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Settings {
	pub currency_auto_exchange: bool,
}

impl Settings {
	fn insert_from_kdl(
		&mut self,
		node: &kdl::KdlNode,
		ctx: &mut NodeContext,
	) -> anyhow::Result<()> {
		match node.get_str_req(ctx.consume_idx())? {
			"currency_auto_exchange" => {
				self.currency_auto_exchange = node.get_bool_req(ctx.consume_idx())?;
			}
			key => {
				return Err(NotInList(key.into(), vec!["currency_auto_exchange"]).into());
			}
		}
		Ok(())
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct SelectedSpells {
	consumed_slots: HashMap<u8, usize>,
	cache_by_caster: HashMap<String, SelectedSpellsData>,
}
#[derive(Clone, PartialEq, Default, Debug)]
pub struct SelectedSpellsData {
	/// The number of rank 0 spells selected.
	pub num_cantrips: usize,
	/// The number of spells selected whose rank is > 0.
	pub num_spells: usize,
	selections: HashMap<SourceId, Spell>,
}
impl FromKDL for SelectedSpells {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let mut consumed_slots = HashMap::new();
		if let Some(node) = node.query_opt("scope() > consumed_slots")? {
			for node in node.query_all("scope() > slot")? {
				let mut ctx = ctx.next_node();
				let slot = node.get_i64_req(ctx.consume_idx())? as u8;
				let consumed = node.get_i64_req(ctx.consume_idx())? as usize;
				consumed_slots.insert(slot, consumed);
			}
		}

		let mut cache_by_caster = HashMap::new();
		for node in node.query_all("scope() > caster")? {
			let mut ctx = ctx.next_node();
			let caster_name = node.get_str_req(ctx.consume_idx())?;
			let mut selection_data = SelectedSpellsData::default();
			for node in node.query_all("scope() > spell")? {
				let spell = Spell::from_kdl(node, &mut ctx.next_node())?;
				selection_data.insert(spell);
			}
			cache_by_caster.insert(caster_name.to_owned(), selection_data);
		}

		Ok(Self {
			consumed_slots,
			cache_by_caster,
		})
	}
}
impl SelectedSpells {
	pub fn insert(&mut self, caster_id: &impl AsRef<str>, spell: Spell) {
		let selected_spells = match self.cache_by_caster.get_mut(caster_id.as_ref()) {
			Some(existing) => existing,
			None => {
				self.cache_by_caster
					.insert(caster_id.as_ref().to_owned(), SelectedSpellsData::default());
				self.cache_by_caster.get_mut(caster_id.as_ref()).unwrap()
			}
		};
		selected_spells.insert(spell);
	}

	pub fn remove(&mut self, caster_id: &impl AsRef<str>, spell_id: &SourceId) {
		let Some(caster_list) = self.cache_by_caster.get_mut(caster_id.as_ref()) else { return; };
		caster_list.remove(spell_id);
	}

	pub fn get(&self, caster_id: &impl AsRef<str>) -> Option<&SelectedSpellsData> {
		self.cache_by_caster.get(caster_id.as_ref())
	}

	pub fn get_spell(&self, caster_id: &impl AsRef<str>, spell_id: &SourceId) -> Option<&Spell> {
		let Some(data) = self.cache_by_caster.get(caster_id.as_ref()) else { return None; };
		let Some(spell) = data.selections.get(spell_id) else { return None; };
		Some(spell)
	}

	pub fn iter_caster_ids(&self) -> impl Iterator<Item = &String> {
		self.cache_by_caster.keys()
	}

	pub fn iter_caster(&self, caster_id: &impl AsRef<str>) -> Option<impl Iterator<Item = &Spell>> {
		let Some(caster) = self.cache_by_caster.get(caster_id.as_ref()) else { return None; };
		Some(caster.selections.values())
	}

	pub fn has_selected(&self, caster_id: &impl AsRef<str>, spell_id: &SourceId) -> bool {
		let Some(data) = self.cache_by_caster.get(caster_id.as_ref()) else { return false; };
		data.selections.contains_key(spell_id)
	}

	pub fn consumed_slots(&self, rank: u8) -> Option<usize> {
		self.consumed_slots.get(&rank).map(|v| *v)
	}

	pub fn set_slots_consumed(&mut self, rank: u8, count: usize) {
		if count == 0 {
			self.consumed_slots.remove(&rank);
			return;
		}
		match self.consumed_slots.get_mut(&rank) {
			None => {
				self.consumed_slots.insert(rank, count);
			}
			Some(slot_count) => {
				*slot_count = count;
			}
		}
	}
}
impl SelectedSpellsData {
	fn insert(&mut self, spell: Spell) {
		match spell.rank {
			0 => self.num_cantrips += 1,
			_ => self.num_spells += 1,
		}
		self.selections.insert(spell.id.clone(), spell);
	}

	fn remove(&mut self, id: &SourceId) {
		if let Some(spell) = self.selections.remove(id) {
			match spell.rank {
				0 => self.num_cantrips -= 1,
				_ => self.num_spells -= 1,
			}
		}
	}

	pub fn len(&self) -> usize {
		self.selections.len()
	}
}
