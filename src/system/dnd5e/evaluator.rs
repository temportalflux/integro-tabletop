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
mod has_attack;
pub use has_attack::*;
mod has_condition;
pub use has_condition::*;
mod has_proficiency;
pub use has_proficiency::*;

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
				crate::system::generics::Registry::default_with_eval::<$eval_ty>()
			);
		};
		($eval_ty:ty, $node_reg:expr) => {
			static NODE_NAME: &str = "evaluator";
			type Target = crate::system::evaluator::Generic<
				<$eval_ty as crate::system::Evaluator>::Context,
				<$eval_ty as crate::system::Evaluator>::Item,
			>;

			fn node_ctx() -> crate::kdl_ext::NodeContext {
				crate::kdl_ext::NodeContext::registry($node_reg)
			}

			fn from_kdl<'doc>(mut node: crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Target> {
				Target::from_kdl(&mut node)
			}

			fn as_kdl<E: crate::system::Evaluator>(data: &E) -> crate::kdl_ext::NodeBuilder {
				let mut node = crate::kdl_ext::NodeBuilder::default();
				node.entry(data.get_id());
				node += data.as_kdl();
				node
			}
		};
	}
	pub(crate) use test_utils;
}
