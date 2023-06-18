mod ability_score;
pub use ability_score::*;

mod action_budget;
pub use action_budget::*;

mod armor_class;
pub use armor_class::*;

mod bonus_damage;
pub use bonus_damage::*;

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

#[cfg(test)]
pub(crate) mod test {
	macro_rules! test_utils {
		($eval_ty:ty) => {
			static NODE_NAME: &str = "mutator";
			type Target =
				crate::utility::GenericMutator<<$eval_ty as crate::utility::Mutator>::Target>;

			fn node_ctx() -> crate::kdl_ext::NodeContext {
				crate::kdl_ext::NodeContext::registry(
					crate::system::core::NodeRegistry::default_with_mut::<$eval_ty>(),
				)
			}

			fn from_kdl(
				node: &::kdl::KdlNode,
				ctx: &mut crate::kdl_ext::NodeContext,
			) -> anyhow::Result<Target> {
				ctx.parse_mutator(node)
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
