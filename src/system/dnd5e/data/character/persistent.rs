use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	path_map::PathMap,
	system::{
		core::SourceId,
		dnd5e::{
			data::{
				character::Character, item::container::Inventory, Ability, Bundle, Class,
				Condition, Rest, Spell,
			},
			SystemComponent,
		},
	},
	utility::{MutatorGroup, NotInList},
};
use enum_map::EnumMap;
use itertools::Itertools;
use multimap::MultiMap;
use serde::{Deserialize, Serialize};
use std::{
	collections::{BTreeMap, HashMap},
	path::Path,
	str::FromStr,
	sync::Arc,
};

mod description;
pub use description::*;

use super::RestEntry;

pub static MAX_SPELL_RANK: u8 = 9;

/// Core character data which is (de)serializable and
/// from which the derived data can be compiled.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Persistent {
	pub id: SourceId,
	pub classes: Vec<Class>,
	pub bundles: Vec<Bundle>,
	pub description: Description,
	pub ability_scores: EnumMap<Ability, u32>,
	pub selected_values: PathMap<String>,
	pub selected_spells: SelectedSpells,
	pub inventory: Inventory,
	pub conditions: Conditions,
	pub hit_points: HitPoints,
	pub inspiration: bool,
	pub settings: Settings,
}
impl MutatorGroup for Persistent {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		for bundle in &self.bundles {
			bundle.set_data_path(parent);
		}
		for group in &self.classes {
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
		{
			// Add the reset data for spell slots (shared by multiple classes when multiclassing).
			// Non-casters will still have this entry, but since they can't cast/don't have any slots,
			// there will be no slots that show up or actual data to reset.
			let (rest, entry) = self.selected_spells.reset_on_rest();
			stats.rest_resets_mut().add(rest, entry);
		}

		for bundle in &self.bundles {
			stats.apply_from(bundle, parent);
		}
		for class in &self.classes {
			stats.apply_from(class, parent);
		}
		stats.apply_from(&self.conditions, parent);
		stats.apply_from(&self.inventory, parent);
	}
}

impl Persistent {
	pub fn add_class(&mut self, class: Class) {
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

	pub fn get_selections_at(&self, path: impl AsRef<Path>) -> Option<&Vec<String>> {
		self.selected_values.get(path.as_ref())
	}

	pub fn get_first_selection(&self, path: impl AsRef<Path>) -> Option<&String> {
		self.get_selections_at(path)
			.map(|all| all.first())
			.flatten()
	}

	pub fn get_first_selection_at<T>(
		&self,
		data_path: impl AsRef<Path>,
	) -> Option<Result<T, <T as FromStr>::Err>>
	where
		T: Clone + 'static + FromStr,
	{
		let selections = self.get_selections_at(data_path);
		selections
			.map(|all| all.first())
			.flatten()
			.map(|selected| T::from_str(&selected))
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

	pub fn remove_selected_value(&mut self, key: impl AsRef<Path>, value: impl Into<String>) {
		let Some(values) = self.selected_values.get_mut(key) else { return; };
		let target: String = value.into();
		values.retain(|value| *value != target);
	}

	pub fn export_as_kdl(&self) -> kdl::KdlDocument {
		let mut doc = kdl::KdlDocument::new();
		doc.nodes_mut().push(self.as_kdl().build("character"));
		doc
	}
}

crate::impl_kdl_node!(Persistent, "character");
#[derive(PartialEq, Serialize, Deserialize)]
pub struct PersistentMetadata {
	pub name: String,
	pub pronouns: Vec<String>,
	pub level: usize,
	pub classes: Vec<String>,
	pub bundles: MultiMap<String, String>,
}
impl SystemComponent for Persistent {
	fn to_metadata(self) -> serde_json::Value {
		let mut level = 0;
		let mut classes = Vec::with_capacity(self.classes.len());
		for class in &self.classes {
			level += class.current_level;
			classes.push(class.name.clone());
		}
		let metadata = PersistentMetadata {
			name: self.description.name.clone(),
			pronouns: self.description.iter_pronouns().cloned().collect(),
			level,
			classes,
			bundles: self
				.bundles
				.iter()
				.map(|bundle| (bundle.category.clone(), bundle.name.clone()))
				.collect(),
		};
		serde_json::json!(metadata)
	}
}
impl FromKDL for Persistent {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.id().clone();

		let description = node.query_req_t::<Description>("scope() > description")?;

		let mut settings = Settings::default();
		for node in &mut node.query_all("scope() > setting")? {
			settings.insert_from_kdl(node)?;
		}

		let mut ability_scores = EnumMap::default();
		for node in &mut node.query_all("scope() > ability")? {
			let ability = node.next_str_req_t::<Ability>()?;
			let score = node.next_i64_req()? as u32;
			ability_scores[ability] = score;
		}

		let hit_points = node.query_req_t::<HitPoints>("scope() > hit_points")?;

		let inspiration = node
			.query_bool_opt("scope() > inspiration", 0)?
			.unwrap_or_default();

		let mut conditions = Conditions::default();
		for condition in node.query_all_t::<Condition>("scope() > condition")? {
			conditions.insert(condition);
		}

		let inventory = node
			.query_opt_t::<Inventory>("scope() > inventory")?
			.unwrap_or_default();

		let selected_spells = node
			.query_opt_t::<SelectedSpells>("scope() > spells")?
			.unwrap_or_default();

		let bundles = node.query_all_t::<Bundle>("scope() > bundle")?;
		let classes = node.query_all_t::<Class>("scope() > class")?;

		let mut selected_values = PathMap::<String>::default();
		if let Some(selections) = node.query_opt("scope() > selections")? {
			for mut node in selections.query_all("scope() > value")? {
				let key_str = node.next_str_req()?;
				let value = node.next_str_req()?.to_owned();
				selected_values.insert(Path::new(key_str), value);
			}
		}

		Ok(Self {
			id,
			description,
			settings,
			ability_scores,
			hit_points,
			inspiration,
			conditions,
			inventory,
			selected_spells,
			bundles,
			classes,
			selected_values,
		})
	}
}
impl AsKdl for Persistent {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_child_t("description", &self.description);
		self.settings.export_as_kdl(&mut node);

		for (ability, score) in self.ability_scores {
			node.push_child(
				NodeBuilder::default()
					.with_entry(ability.long_name())
					.with_entry(score as i64)
					.build("ability"),
			);
		}

		node.push_child_t("hit_points", &self.hit_points);
		node.push_child_t("inspiration", &self.inspiration);

		node.push_child_opt_t("inventory", &self.inventory);
		node.push_child_opt_t("spells", &self.selected_spells);

		for bundle in &self.bundles {
			node.push_child_opt_t("bundle", bundle);
		}
		for class in &self.classes {
			node.push_child_opt_t("class", class);
		}

		node.push_child_opt({
			let mut node = NodeBuilder::default();
			for (path, value) in self.selected_values.as_vec() {
				node.push_child(
					NodeBuilder::default()
						.with_entry({
							let path_str = path.display().to_string();
							path_str.replace("\\", "/")
						})
						.with_entry(value.clone())
						.build("value"),
				);
			}
			node.build("selections")
		});

		node
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
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
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
impl AsKdl for HitPoints {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_child_t("current", &self.current);
		node.push_child_t("temp", &self.temp);
		node.push_child_t("failure_saves", &self.failure_saves);
		node.push_child_t("success_saves", &self.success_saves);
		node
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

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Condition> {
		self.by_id.values_mut().chain(self.custom.iter_mut())
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
	fn insert_from_kdl<'doc>(
		&mut self,
		node: &mut crate::kdl_ext::NodeReader<'doc>,
	) -> anyhow::Result<()> {
		match node.next_str_req()? {
			"currency_auto_exchange" => {
				self.currency_auto_exchange = node.next_bool_req()?;
			}
			key => {
				return Err(NotInList(key.into(), vec!["currency_auto_exchange"]).into());
			}
		}
		Ok(())
	}

	fn export_as_kdl(&self, nodes: &mut NodeBuilder) {
		nodes.push_child(
			NodeBuilder::default()
				.with_entry("currency_auto_exchange")
				.with_entry(self.currency_auto_exchange)
				.build("setting"),
		);
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct SelectedSpells {
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
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut cache_by_caster = HashMap::new();
		for node in &mut node.query_all("scope() > caster")? {
			let caster_name = node.next_str_req()?;
			let mut selection_data = SelectedSpellsData::default();
			for mut node in &mut node.query_all("scope() > spell")? {
				let spell = Spell::from_kdl(&mut node)?;
				selection_data.insert(spell);
			}
			cache_by_caster.insert(caster_name.to_owned(), selection_data);
		}

		Ok(Self { cache_by_caster })
	}
}
impl AsKdl for SelectedSpells {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		// Casters
		let iter_casters = self.cache_by_caster.iter();
		let iter_casters = iter_casters.sorted_by_key(|(name, _)| *name);
		for (caster_name, selected_spells) in iter_casters {
			if selected_spells.selections.is_empty() {
				continue;
			}
			let mut node_caster = NodeBuilder::default();

			node_caster.push_entry(caster_name.clone());

			let iter_spells = selected_spells.selections.values();
			let iter_spells =
				iter_spells.sorted_by(|a, b| a.rank.cmp(&b.rank).then(a.name.cmp(&b.name)));
			for spell in iter_spells {
				node_caster.push_child_t("spell", spell);
			}

			node.push_child(node_caster.build("caster"));
		}
		node
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

	pub fn consumed_slots_path(&self, rank: u8) -> std::path::PathBuf {
		Path::new("SpellSlots").join(rank.to_string())
	}

	pub fn reset_on_rest(&self) -> (Rest, RestEntry) {
		let data_paths = (1..=MAX_SPELL_RANK)
			.into_iter()
			.map(|rank| self.consumed_slots_path(rank))
			.collect::<Vec<_>>();
		let entry = RestEntry {
			restore_amount: None,
			data_paths,
			source: Path::new("Standard Spellcasting Slots").to_owned(),
		};
		(Rest::Long, entry)
	}
}
impl SelectedSpellsData {
	fn insert(&mut self, spell: Spell) {
		match spell.rank {
			0 => self.num_cantrips += 1,
			_ => self.num_spells += 1,
		}
		self.selections.insert(spell.id.unversioned(), spell);
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

#[cfg(test)]
mod test_hit_points {
	use super::*;
	use crate::kdl_ext::test_utils::*;

	static NODE_NAME: &str = "hit_points";

	#[test]
	fn kdl() -> anyhow::Result<()> {
		let doc = "
			|hit_points {
			|    current 30
			|    temp 5
			|    failure_saves 1
			|    success_saves 2
			|}
		";
		let data = HitPoints {
			current: 30,
			temp: 5,
			failure_saves: 1,
			success_saves: 2,
		};
		assert_eq_fromkdl!(HitPoints, doc, data);
		assert_eq_askdl!(&data, doc);
		Ok(())
	}
}
