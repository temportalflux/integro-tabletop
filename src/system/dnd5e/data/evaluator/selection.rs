use crate::{system::dnd5e::data::character::Character, utility::Evaluator};
use std::{
	collections::BTreeMap,
	fmt::Debug,
	path::{Component, PathBuf},
	str::FromStr,
};

/// Maps some selection value `K`, at selector `selector_path`, to a evaluation value `V`.
#[derive(Clone, PartialEq, Debug)]
pub struct BySelection<K, V> {
	pub selector_path: PathBuf,
	pub map: BTreeMap<K, V>,
}

impl<K, V, const N: usize> From<(PathBuf, [(K, V); N])> for BySelection<K, V>
where
	K: Ord,
{
	fn from((path, values): (PathBuf, [(K, V); N])) -> Self {
		Self {
			selector_path: path,
			map: BTreeMap::from(values),
		}
	}
}

impl<K, V> crate::utility::TraitEq for BySelection<K, V>
where
	K: 'static + PartialEq,
	V: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl<K, V> Evaluator for BySelection<K, V>
where
	K: 'static + Clone + Send + Sync + Debug + PartialEq + FromStr + Ord,
	V: 'static + Clone + Send + Sync + Debug + PartialEq + Default,
{
	type Context = Character;
	type Item = V;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let selection_path = {
			let path = state.source_path().join(&self.selector_path);
			// Lexical path resolution (resolve `./` and `../` without accessing filesystem)
			path.components().fold(PathBuf::new(), |mut path, comp| {
				match comp {
					Component::CurDir => {}
					Component::ParentDir => {
						path.pop();
					}
					_ => path.push(comp.as_os_str()),
				}
				path
			})
		};
		match state.get_first_selection_at::<K>(&selection_path) {
			Some(Ok(key)) => self.map.get(&key).cloned().unwrap_or_default(),
			Some(Err(_)) => {
				// TODO: Emit warning that the selector value could not be parsed
				V::default()
			}
			None => {
				// TODO: Emit warning that the selection is missing
				V::default()
			}
		}
	}
}
