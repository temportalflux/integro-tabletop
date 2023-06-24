use crate::system::{
	core::SourceId,
	dnd5e::data::{
		character::{Character, ObjectCacheProvider, Persistent},
		spell::Spell,
	},
};
use multimap::MultiMap;
use std::{
	collections::{BTreeMap, HashMap},
	path::PathBuf,
};

mod caster;
pub use caster::*;
mod cantrips;
pub use cantrips::*;
mod entry;
pub use entry::*;
mod filter;
pub use filter::*;
mod slots;
pub use slots::*;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Spellcasting {
	// Output goals:
	// - cantrip capacity
	// - cantrips prepared
	// - spell slot map (rank to slot capacity and number used)
	// - spell capacity (number of spells that can be prepared/known)
	// - spells prepared (or known)
	casters: HashMap<String, Caster>,
	always_prepared: HashMap<SourceId, AlwaysPreparedSpell>,
	ritual_spells: RitualSpellCache,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AlwaysPreparedSpell {
	pub spell: Option<Spell>,
	pub entries: HashMap<PathBuf, SpellEntry>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct RitualSpellCache {
	pub spells: HashMap<SourceId, Spell>,
	pub caster_lists: MultiMap<String, SourceId>,
}

impl Spellcasting {
	pub fn add_caster(&mut self, caster: Caster) {
		self.casters.insert(caster.name().clone(), caster);
	}

	pub fn add_prepared(&mut self, spell_id: &SourceId, entry: SpellEntry) {
		if !self.always_prepared.contains_key(&spell_id) {
			self.always_prepared
				.insert(spell_id.clone(), AlwaysPreparedSpell::default());
		}
		self.always_prepared
			.get_mut(spell_id)
			.unwrap()
			.entries
			.insert(entry.source.clone(), entry);
	}

	pub async fn fetch_spell_objects(
		&mut self,
		provider: ObjectCacheProvider,
		persistent: &Persistent,
	) -> anyhow::Result<()> {
		self.fetch_always_prepared(&provider).await?;
		self.ritual_spells = self.fetch_rituals(&provider, persistent).await?;
		Ok(())
	}

	async fn fetch_always_prepared(
		&mut self,
		provider: &ObjectCacheProvider,
	) -> anyhow::Result<()> {
		for (id, spell_entry) in &mut self.always_prepared {
			spell_entry.spell = provider
				.database
				.get_typed_entry::<Spell>(id.clone(), provider.system_depot.clone())
				.await?;
		}
		Ok(())
	}

	async fn fetch_rituals(
		&self,
		provider: &ObjectCacheProvider,
		persistent: &Persistent,
	) -> anyhow::Result<RitualSpellCache> {
		use crate::database::app::Criteria;
		use crate::system::{core::System, dnd5e::DnD5e};
		use futures_util::StreamExt;

		// TODO: For wizards, this should check the spell source instead of always checking the database for spells.

		let mut caster_query_criteria = Vec::new();
		let mut caster_filters = HashMap::new();
		for caster in self.iter_casters() {
			let Some(ritual_capability) = &caster.ritual_capability else { continue; };
			if !ritual_capability.available_spells {
				continue;
			}

			let mut filter = caster.spell_filter(persistent);
			// each spell the filter matches must be a ritual
			filter.ritual = Some(true);

			caster_query_criteria.push(filter.as_criteria());
			caster_filters.insert(caster.name(), filter);
		}
		let criteria = Criteria::Any(caster_query_criteria);

		let db = provider.database.clone();
		let depot = provider.system_depot.clone();
		let query_async = db.query_typed::<Spell>(DnD5e::id(), depot, Some(criteria.into()));
		let query_stream_res = query_async.await;
		let mut query_stream = query_stream_res.map_err(crate::database::Error::from)?;

		let mut ritual_spell_cache = HashMap::new();
		let mut caster_ritual_list_cache = MultiMap::new();
		while let Some(spell) = query_stream.next().await {
			for (caster_id, filter) in &caster_filters {
				if filter.spell_matches(&spell) {
					caster_ritual_list_cache.insert((*caster_id).clone(), spell.id.unversioned());
				}
			}
			ritual_spell_cache.insert(spell.id.unversioned(), spell);
		}

		Ok(RitualSpellCache {
			spells: ritual_spell_cache,
			caster_lists: caster_ritual_list_cache,
		})
	}

	pub fn iter_ritual_spells(&self) -> impl Iterator<Item = (&String, &Spell, &SpellEntry)> + '_ {
		let iter = self.casters.iter();
		let iter = iter.filter_map(|(caster_id, caster)| {
			match self.ritual_spells.caster_lists.get_vec(caster_id) {
				Some(spell_ids) => Some((caster_id, &caster.spell_entry, spell_ids)),
				None => None,
			}
		});
		let iter = iter.map(|(caster_id, caster_spell_entry, spell_ids)| {
			let iter = spell_ids.iter();
			let iter = iter.filter_map(|spell_id| self.ritual_spells.spells.get(spell_id));
			iter.map(move |spell| (caster_id, spell, caster_spell_entry))
		});
		iter.flatten()
	}

	pub fn cantrip_capacity(&self, persistent: &Persistent) -> Vec<(usize, &Restriction)> {
		let mut total_capacity = Vec::new();
		for (_id, caster) in &self.casters {
			let value = caster.cantrip_capacity(persistent);
			if value > 0 {
				total_capacity.push((value, &caster.restriction));
			}
		}
		total_capacity
	}

	/// Returns the spell slots the character has to cast from.
	/// If there are multiple caster features, the spell slots are determined from multiclassing rules.
	pub fn spell_slots(&self, character: &Character) -> Option<BTreeMap<u8, usize>> {
		// https://www.dndbeyond.com/sources/basic-rules/customization-options#MulticlassSpellcaster
		lazy_static::lazy_static! {
			static ref MULTICLASS_SLOTS: BTreeMap<usize, BTreeMap<u8, usize>> = BTreeMap::from([
				( 1, [ (1, 2) ].into()),
				( 2, [ (1, 3) ].into()),
				( 3, [ (1, 4), (2, 2) ].into()),
				( 4, [ (1, 4), (2, 3) ].into()),
				( 5, [ (1, 4), (2, 3), (3, 2) ].into()),
				( 6, [ (1, 4), (2, 3), (3, 3) ].into()),
				( 7, [ (1, 4), (2, 3), (3, 3), (4, 1) ].into()),
				( 8, [ (1, 4), (2, 3), (3, 3), (4, 2) ].into()),
				( 9, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 1) ].into()),
				(10, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2) ].into()),
				(11, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1) ].into()),
				(12, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1) ].into()),
				(13, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1) ].into()),
				(14, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1) ].into()),
				(15, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1), (8, 1) ].into()),
				(16, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1), (8, 1) ].into()),
				(17, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1), (8, 1), (9, 1) ].into()),
				(18, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 3), (6, 1), (7, 1), (8, 1), (9, 1) ].into()),
				(19, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 3), (6, 2), (7, 1), (8, 1), (9, 1) ].into()),
				(20, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 3), (6, 2), (7, 2), (8, 1), (9, 1) ].into()),
			]);
		}

		if self.casters.is_empty() {
			return None;
		}

		let (total_caster_level, slots_by_level) = if self.casters.len() == 1 {
			let (_id, caster) = self.casters.iter().next().unwrap();
			let current_level = character.level(Some(&caster.class_name));
			(current_level, &caster.slots.slots_capacity)
		} else {
			let mut levels = 0;
			for (_id, caster) in &self.casters {
				let current_level = character.level(Some(&caster.class_name));
				levels += match caster.slots.multiclass_half_caster {
					false => current_level,
					true => current_level / 2,
				};
			}
			(levels, &*MULTICLASS_SLOTS)
		};

		for (level, ranks) in slots_by_level.iter().rev() {
			if *level <= total_caster_level {
				return Some(ranks.clone());
			}
		}

		None
	}

	pub fn prepared_spells(&self) -> &HashMap<SourceId, AlwaysPreparedSpell> {
		&self.always_prepared
	}

	pub fn has_casters(&self) -> bool {
		!self.casters.is_empty()
	}

	pub fn get_caster(&self, id: &str) -> Option<&Caster> {
		self.casters.get(id)
	}

	pub fn iter_casters(&self) -> impl Iterator<Item = &Caster> {
		self.casters.iter().map(|(_id, caster)| caster)
	}

	pub fn get_ritual(&self, spell_id: &SourceId) -> Option<&Spell> {
		self.ritual_spells.spells.get(spell_id)
	}
}
