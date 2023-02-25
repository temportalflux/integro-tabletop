use crate::utility::RcEvaluator;
use self::data::character::{Character, Persistent};

pub mod components;
pub mod content;
pub mod data;

pub type BoxedCriteria = RcEvaluator<Persistent, Result<(), String>>;
pub type BoxedEvaluator<V> = RcEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::RcMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;
