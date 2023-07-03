mod ability_score;
pub use ability_score::*;

mod action_budget;
pub use action_budget::*;

mod add_bundle;
pub use add_bundle::*;

mod apply_if;
pub use apply_if::*;

mod armor_class;
pub use armor_class::*;

mod bonus;
pub use bonus::*;

mod description;
pub use description::*;

mod defense;
pub use defense::*;

mod feature;
pub use feature::*;

mod flag;
pub use flag::*;

mod hit_points;
pub use hit_points::*;

mod level;
pub use level::*;

mod modifier;
pub use modifier::*;

mod proficient;
pub use proficient::*;

mod pick_n;
pub use pick_n::*;

mod sense;
pub use sense::*;

mod speed;
pub use speed::*;

mod spellcasting;
pub use spellcasting::*;

mod starting_equipment;
pub use starting_equipment::*;

#[cfg(test)]
pub(crate) mod test {
	macro_rules! test_utils {
		($mut_ty:ty) => {
			test_utils!(
				$mut_ty,
				crate::system::core::NodeRegistry::default_with_mut::<$mut_ty>()
			);
		};
		($mut_ty:ty, $node_reg:expr) => {
			static NODE_NAME: &str = "mutator";
			type Target =
				crate::utility::GenericMutator<<$mut_ty as crate::utility::Mutator>::Target>;

			fn node_ctx() -> crate::kdl_ext::NodeContext {
				crate::kdl_ext::NodeContext::registry($node_reg)
			}

			fn from_kdl<'doc>(node: crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Target> {
				node.parse_mutator()
			}

			fn as_kdl<E: crate::utility::Mutator>(data: &E) -> crate::kdl_ext::NodeBuilder {
				crate::kdl_ext::NodeBuilder::default()
					.with_entry(data.get_id())
					.with_extension(data.as_kdl())
			}
		};
	}
	pub(crate) use test_utils;
}
