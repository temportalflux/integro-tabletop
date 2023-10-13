use super::Entry;
use crate::{database::Cursor, kdl_ext::NodeContext, system::core::NodeRegistry};
use futures_util::StreamExt;
use kdlize::FromKdl;
use std::{pin::Pin, sync::Arc, task::Poll};

#[derive(Debug, Clone, PartialEq)]
pub enum Criteria {
	/// Passes if the value being evaluated is equal to an expected value.
	Exact(serde_json::Value),
	/// Passes if the value being evaluated does not pass the provided criteria.
	Not(Box<Criteria>),
	/// Passes if the value being evaluated:
	/// 1. Is a string
	/// 2. Contains the provided substring
	ContainsSubstring(String),
	/// Passes if the value being evaluated:
	/// 1. Is an object/map
	/// 2. Contains the provided key
	/// 3. The value at the provided key passes the provided criteria
	ContainsProperty(String, Box<Criteria>),
	/// Passes if the value being evaluated:
	/// 1. Is an array
	/// 2. The criteria matches against any of the contents
	ContainsElement(Box<Criteria>),
	/// Passes if the value being evaluated passes all of the provided criteria.
	All(Vec<Criteria>),
	/// Passes if the value being evaluated passes any of the provided criteria.
	Any(Vec<Criteria>),
}

impl Criteria {
	/// Converts `value` into a `serde_json::Value`, and returns the `Exact` enum with that value.
	pub fn exact<T: Into<serde_json::Value>>(value: T) -> Self {
		Self::Exact(value.into())
	}

	/// Returns the `Not` enum with the boxed value of `criteria`.
	pub fn not(criteria: Self) -> Self {
		Self::Not(criteria.into())
	}

	/// Returns the `ContainsSubstring` enum with the provided string.
	pub fn contains_substr(str: String) -> Self {
		Self::ContainsSubstring(str)
	}

	/// Returns the `ContainsProperty` enum with the provided string key and the boxed value of `criteria`.
	pub fn contains_prop(key: impl Into<String>, criteria: Self) -> Self {
		Self::ContainsProperty(key.into(), criteria.into())
	}

	/// Returns the `ContainsElement` enum with the boxed value of `criteria`.
	pub fn contains_element(criteria: Self) -> Self {
		Self::ContainsElement(criteria.into())
	}

	/// Returns the `All` enum with the value of `items` being collected as a `Vec`.
	pub fn all<I: IntoIterator<Item = Criteria>>(items: I) -> Self {
		Self::All(items.into_iter().collect())
	}

	/// Returns the `Any` enum with the value of `items` being collected as a `Vec`.
	pub fn any<I: IntoIterator<Item = Criteria>>(items: I) -> Self {
		Self::Any(items.into_iter().collect())
	}

	pub fn is_relevant(&self, value: &serde_json::Value) -> bool {
		match self {
			Self::Exact(expected) => value == expected,
			Self::Not(criteria) => !criteria.is_relevant(value),
			Self::ContainsSubstring(substring) => {
				let serde_json::Value::String(value) = value else {
					return false;
				};
				value.to_lowercase().contains(&substring.to_lowercase())
			}
			Self::ContainsProperty(key, criteria) => {
				let serde_json::Value::Object(map) = value else {
					return false;
				};
				let Some(value) = map.get(key) else {
					return false;
				};
				criteria.is_relevant(value)
			}
			Self::ContainsElement(criteria) => {
				let serde_json::Value::Array(value_list) = value else {
					return false;
				};
				for value in value_list {
					if criteria.is_relevant(value) {
						return true;
					}
				}
				false
			}
			Self::All(criteria) => {
				for criteria in criteria {
					if !criteria.is_relevant(value) {
						return false;
					}
				}
				true
			}
			Self::Any(criteria) => {
				for criteria in criteria {
					if criteria.is_relevant(value) {
						return true;
					}
				}
				false
			}
		}
	}
}

// TODO: Tests for criteria against specific json values

pub struct Query {
	pub cursor: Cursor<Entry>,
	pub criteria: Option<Box<Criteria>>,
}

impl futures_util::stream::Stream for Query {
	type Item = Entry;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
		loop {
			let Poll::Ready(entry) = self.cursor.poll_next_unpin(cx) else {
				return Poll::Pending;
			};
			let Some(entry) = entry else {
				return Poll::Ready(None);
			};
			if let Some(criteria) = &self.criteria {
				if !criteria.is_relevant(&entry.metadata) {
					continue;
				}
			}
			return Poll::Ready(Some(entry));
		}
	}
}

pub struct QueryDeserialize<Output> {
	#[allow(dead_code)]
	pub db: super::Database,
	pub query: Query,
	pub node_reg: Arc<NodeRegistry>,
	pub marker: std::marker::PhantomData<Output>,
}
impl<Output> futures_util::stream::Stream for QueryDeserialize<Output>
where
	Output: FromKdl<NodeContext> + Unpin,
{
	type Item = Output;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
		loop {
			// Get the next database entry based on the query and underlying cursor
			let Poll::Ready(entry) = self.query.poll_next_unpin(cx) else {
				return Poll::Pending;
			};
			let Some(entry) = entry else {
				return Poll::Ready(None);
			};
			let Some(value) = entry.parse_kdl::<Output>(self.node_reg.clone()) else {
				continue;
			};
			// we found a sucessful value! we can return it
			return Poll::Ready(Some(value));
		}
	}
}
impl<Output> QueryDeserialize<Output>
where
	Output: FromKdl<NodeContext> + Unpin,
{
	pub async fn first_n(mut self, limit: Option<usize>) -> Vec<Output> {
		let mut items = Vec::new();
		while let Some(item) = self.next().await {
			items.push(item);
			if let Some(limit) = &limit {
				if items.len() >= *limit {
					break;
				}
			}
		}
		items
	}

	pub async fn all(mut self) -> Vec<Output> {
		let mut items = Vec::new();
		while let Some(item) = self.next().await {
			items.push(item);
		}
		items
	}
}
