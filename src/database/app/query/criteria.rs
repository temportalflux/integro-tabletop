use crate::database::Entry;

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
	/// 1. Is an object/map
	/// 2. Does not have the provided key
	MissingProperty(String),
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

	/// Returns the `MissingProperty` enum with the provided string key.
	pub fn missing_prop(key: impl Into<String>) -> Self {
		Self::MissingProperty(key.into())
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
			Self::MissingProperty(key) => {
				let serde_json::Value::Object(map) = value else {
					return false;
				};
				!map.contains_key(key)
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
				if criteria.is_empty() {
					return true;
				}
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

impl Criteria {
	pub fn as_predicate(&self) -> impl FnMut(&Entry) -> futures::future::Ready<bool> + '_ {
		|entry: &Entry| futures::future::ready(self.is_relevant(&entry.metadata))
	}

	pub fn into_predicate(self) -> impl FnMut(&Entry) -> futures::future::Ready<bool> {
		move |entry: &Entry| futures::future::ready(self.is_relevant(&entry.metadata))
	}
}

// TODO: Tests for criteria against specific json values
