use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModuleId {
	Local {
		name: String,
	},
	Github {
		user_org: String,
		repository: String,
	},
}
impl Default for ModuleId {
	fn default() -> Self {
		Self::Local {
			name: Default::default(),
		}
	}
}
impl ToString for ModuleId {
	fn to_string(&self) -> String {
		match &self {
			ModuleId::Local { name } => name.clone(),
			ModuleId::Github {
				user_org,
				repository,
			} => format!("{user_org}/{repository}"),
		}
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId {
	pub module: Option<ModuleId>,
	pub system: Option<String>,
	pub path: PathBuf,
	pub version: Option<String>,
	pub node_idx: usize,
}

impl SourceId {
	pub fn set_basis(&mut self, other: &Self, include_version: bool) {
		if self.module.is_none() {
			self.module = other.module.clone();
		}
		if self.system.is_none() {
			self.system = other.system.clone();
		}
		if include_version && self.version.is_none() {
			self.version = other.version.clone();
		}
	}

	pub fn with_basis(mut self, other: &Self, include_version: bool) -> Self {
		self.set_basis(other, include_version);
		self
	}

	pub fn ref_id(&self) -> String {
		let prefix = match &self.module {
			None => String::default(),
			Some(ModuleId::Local { name }) => format!("{name}-"),
			Some(ModuleId::Github {
				user_org,
				repository,
			}) => format!("{user_org}_{repository}-"),
		};
		let name = self.path.file_stem().unwrap().to_str().unwrap();
		format!("{prefix}{name}")
	}

	pub fn unversioned(&self) -> Self {
		self.clone().into_unversioned()
	}

	pub fn into_unversioned(mut self) -> Self {
		self.version = None;
		self
	}
}

impl ToString for SourceId {
	fn to_string(&self) -> String {
		let mut comps = Vec::new();
		if let Some(id) = &self.module {
			comps.push(match id {
				ModuleId::Local { name } => {
					format!("local://{name}")
				}
				ModuleId::Github {
					user_org,
					repository,
				} => {
					format!("github://{user_org}:{repository}")
				}
			});
		}
		if let Some(system) = &self.system {
			comps.push(format!("@{system}"));
		}
		if !comps.is_empty() {
			comps.push("/".into());
		}
		comps.push(self.path.display().to_string().replace("\\", "/"));
		if let Some(version) = &self.version {
			comps.push(format!("?version={version}"));
		}
		if self.node_idx > 0 {
			comps.push(format!("#{}", self.node_idx));
		}
		comps.join("")
	}
}

impl FromStr for SourceId {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let url = match url::Url::from_str(s) {
			Ok(url) => url,
			Err(url::ParseError::RelativeUrlWithoutBase) => {
				let path = PathBuf::from_str(s)?;
				return Ok(Self {
					path,
					..Default::default()
				});
			}
			Err(err) => return Err(err.into()),
		};

		let module_name = url.username().to_owned();
		let system = url
			.host_str()
			.ok_or(SourceIdParseError::MissingSystemId)?
			.to_owned();

		let module = match url.scheme() {
			"local" => ModuleId::Local { name: module_name },
			"github" => ModuleId::Github {
				user_org: module_name,
				repository: url
					.password()
					.ok_or(SourceIdParseError::MissingRepository)?
					.to_string(),
			},
			scheme => {
				return Err(
					SourceIdParseError::UnrecognizedModuleService(scheme.to_owned()).into(),
				);
			}
		};
		let mut path = PathBuf::from_str(url.path())?;
		if url.scheme() != "file" {
			path = path.strip_prefix("/")?.to_owned();
		}
		let version = match url.query_pairs().next() {
			Some((key, value)) if key == "version" => Some(value.to_string()),
			_ => None,
		};
		let node_idx = match url.fragment() {
			None => None,
			Some(fragment) => Some(fragment.parse::<usize>()?),
		}
		.unwrap_or_default();
		Ok(Self {
			system: Some(system),
			module: Some(module),
			path,
			version,
			node_idx,
		})
	}
}

#[derive(thiserror::Error, Debug)]
enum SourceIdParseError {
	#[error("Missing system id in host field of url")]
	MissingSystemId,
	#[error("Missing repository name in password field of url")]
	MissingRepository,
	#[error("Unrecognized module service {0}")]
	UnrecognizedModuleService(String),
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn relative_path() -> anyhow::Result<()> {
		let src = "items/trinket.kdl";
		let source_id = SourceId::from_str(src)?;
		assert_eq!(
			source_id,
			SourceId {
				module: None,
				system: None,
				path: "items/trinket.kdl".into(),
				version: None,
				node_idx: 0,
			}
		);
		Ok(())
	}

	#[test]
	fn file_no_module() -> anyhow::Result<()> {
		let src = "local://homebrew@dnd5e/items/trinket.kdl#32";
		let source_id = SourceId::from_str(src)?;
		assert_eq!(
			source_id,
			SourceId {
				module: Some(ModuleId::Local {
					name: "homebrew".into()
				}),
				system: Some("dnd5e".into()),
				path: "items/trinket.kdl".into(),
				version: None,
				node_idx: 32,
			}
		);
		Ok(())
	}

	#[test]
	fn file_to_str() {
		let source = SourceId {
			module: Some(ModuleId::Local {
				name: "homebrew".into(),
			}),
			system: Some("dnd5e".into()),
			path: "items/trinket.kdl".into(),
			version: None,
			node_idx: 0,
		};
		assert_eq!(
			source.to_string(),
			"local://homebrew@dnd5e/items/trinket.kdl"
		);
	}

	#[test]
	fn github() -> anyhow::Result<()> {
		let src = "github://ghuser:homebrew@dnd5e/items/trinket.kdl?version=4b37d0e2a#5";
		let source_id = SourceId::from_str(src)?;
		assert_eq!(
			source_id,
			SourceId {
				module: Some(ModuleId::Github {
					user_org: "ghuser".into(),
					repository: "homebrew".into()
				}),
				system: Some("dnd5e".into()),
				path: "items/trinket.kdl".into(),
				version: Some("4b37d0e2a".into()),
				node_idx: 5,
			}
		);
		Ok(())
	}

	#[test]
	fn github_to_str() {
		let source = SourceId {
			module: Some(ModuleId::Github {
				user_org: "ghuser".into(),
				repository: "homebrew".into(),
			}),
			system: Some("dnd5e".into()),
			path: "items/trinket.kdl".into(),
			version: Some("4b37d0e2a".into()),
			node_idx: 7,
		};
		assert_eq!(
			source.to_string(),
			"github://ghuser:homebrew@dnd5e/items/trinket.kdl?version=4b37d0e2a#7"
		);
	}

	#[test]
	fn path_to_str() {
		let source = SourceId {
			module: None,
			system: None,
			path: "items/trinket.kdl".into(),
			version: None,
			node_idx: 0,
		};
		assert_eq!(source.to_string(), "items/trinket.kdl");
	}

	#[test]
	fn rebased() -> anyhow::Result<()> {
		let basis =
			SourceId::from_str("local://module-name@mysystem/item/gear.kdl?version=e812da2c")?;
		let mut relative = SourceId::from_str("feat/initiate.kdl")?;
		relative.set_basis(&basis, true);
		let expected =
			SourceId::from_str("local://module-name@mysystem/feat/initiate.kdl?version=e812da2c")?;
		assert_eq!(relative, expected);
		Ok(())
	}
}
