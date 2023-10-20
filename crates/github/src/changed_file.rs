#[derive(Debug, Clone, PartialEq)]
pub struct ChangedFile {
	pub path: String,
	pub file_id: String,
	pub status: ChangedFileStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangedFileStatus {
	Added,
	Removed,
	Modified,
	Renamed,
	Copied,
	Changed,
	Unchanged,
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid file status {0:?}")]
pub struct InvalidChangedFileStatus(String);

impl std::str::FromStr for ChangedFileStatus {
	type Err = InvalidChangedFileStatus;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"added" => Ok(Self::Added),
			"removed" => Ok(Self::Removed),
			"modified" => Ok(Self::Modified),
			"renamed" => Ok(Self::Renamed),
			"copied" => Ok(Self::Copied),
			"changed" => Ok(Self::Changed),
			"unchanged" => Ok(Self::Unchanged),
			_ => Err(InvalidChangedFileStatus(s.to_owned())),
		}
	}
}
