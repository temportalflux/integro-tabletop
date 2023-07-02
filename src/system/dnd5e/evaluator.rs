mod get_ability;
pub use get_ability::*;
mod get_prof_bonus;
pub use get_prof_bonus::*;
mod get_hit_points;
pub use get_hit_points::*;
mod get_level;
pub use get_level::*;

mod has_armor;
pub use has_armor::*;
mod has_condition;
pub use has_condition::*;
mod has_proficiency;
pub use has_proficiency::*;
mod has_weapon;
pub use has_weapon::*;

mod logic;
pub use logic::*;

mod math;
pub use math::*;

#[cfg(test)]
pub(crate) mod test {
	macro_rules! test_utils {
		($eval_ty:ty) => {
			test_utils!(
				$eval_ty,
				crate::system::core::NodeRegistry::default_with_eval::<$eval_ty>()
			);
		};
		($eval_ty:ty, $node_reg:expr) => {
			static NODE_NAME: &str = "evaluator";
			type Target = crate::utility::GenericEvaluator<
				<$eval_ty as crate::utility::Evaluator>::Context,
				<$eval_ty as crate::utility::Evaluator>::Item,
			>;

			fn node_ctx() -> crate::kdl_ext::NodeContext {
				crate::kdl_ext::NodeContext::registry($node_reg)
			}

			fn from_kdl(
				node: &::kdl::KdlNode,
				ctx: &mut crate::kdl_ext::NodeContext,
			) -> anyhow::Result<Target> {
				ctx.parse_evaluator_inline(node)
			}

			fn as_kdl<E: crate::utility::Evaluator>(data: &E) -> crate::kdl_ext::NodeBuilder {
				crate::kdl_ext::NodeBuilder::default()
					.with_entry(data.get_id())
					.with_extension(data.as_kdl())
			}
		};
	}
	pub(crate) use test_utils;
}
