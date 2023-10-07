use crate::error::InvalidValueType;

pub trait ValueExt {
	/// Returns the value of the entry.
	/// If the value is not a bool, an error is returned.
	fn as_bool_req(&self) -> Result<bool, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a i64, an error is returned.
	fn as_i64_req(&self) -> Result<i64, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a f64, an error is returned.
	fn as_f64_req(&self) -> Result<f64, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a string, an error is returned.
	fn as_str_req(&self) -> Result<&str, InvalidValueType>;
}

impl ValueExt for kdl::KdlValue {
	fn as_bool_req(&self) -> Result<bool, InvalidValueType> {
		Ok(self
			.as_bool()
			.ok_or(InvalidValueType(self.clone(), "bool"))?)
	}

	fn as_i64_req(&self) -> Result<i64, InvalidValueType> {
		Ok(self.as_i64().ok_or(InvalidValueType(self.clone(), "i64"))?)
	}

	fn as_f64_req(&self) -> Result<f64, InvalidValueType> {
		Ok(self.as_f64().ok_or(InvalidValueType(self.clone(), "f64"))?)
	}

	fn as_str_req(&self) -> Result<&str, InvalidValueType> {
		Ok(self
			.as_string()
			.ok_or(InvalidValueType(self.clone(), "string"))?)
	}
}

impl ValueExt for kdl::KdlEntry {
	fn as_bool_req(&self) -> Result<bool, InvalidValueType> {
		self.value().as_bool_req()
	}

	fn as_i64_req(&self) -> Result<i64, InvalidValueType> {
		self.value().as_i64_req()
	}

	fn as_f64_req(&self) -> Result<f64, InvalidValueType> {
		self.value().as_f64_req()
	}

	fn as_str_req(&self) -> Result<&str, InvalidValueType> {
		self.value().as_str_req()
	}
}
