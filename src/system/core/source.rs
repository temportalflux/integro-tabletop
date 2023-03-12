use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModuleId {
	Local {
		name: String,
	},
	Github {
		user_org: String,
		repository: String,
	},
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId {
	pub module: ModuleId,
	pub system: String,
	pub path: PathBuf,
	pub version: Option<String>,
	pub node_idx: usize,
}

impl ToString for SourceId {
	fn to_string(&self) -> String {
		let url_str = match &self.module {
			ModuleId::Local { name } => {
				format!("local://{name}")
			}
			ModuleId::Github {
				user_org,
				repository,
			} => {
				format!("github://{user_org}:{repository}")
			}
		};
		let mut url_str =
			format!("{url_str}@{}/{}", self.system, self.path.display()).replace("\\", "/");
		if let Some(query) = self.version.as_ref().map(|v| format!("?version={v}")) {
			url_str.push_str(&query);
		}
		if self.node_idx > 0 {
			url_str.push_str(&format!("#{}", self.node_idx));
		}
		url_str
	}
}

impl FromStr for SourceId {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let url = url::Url::from_str(s)?;

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
			system,
			module,
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
	fn file_no_module() -> anyhow::Result<()> {
		let src = "local://homebrew@dnd5e/items/trinket.kdl#32";
		let source_id = SourceId::from_str(src)?;
		assert_eq!(
			source_id,
			SourceId {
				module: ModuleId::Local {
					name: "homebrew".into()
				},
				system: "dnd5e".into(),
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
			module: ModuleId::Local {
				name: "homebrew".into(),
			},
			system: "dnd5e".into(),
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
				module: ModuleId::Github {
					user_org: "ghuser".into(),
					repository: "homebrew".into()
				},
				system: "dnd5e".into(),
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
			module: ModuleId::Github {
				user_org: "ghuser".into(),
				repository: "homebrew".into(),
			},
			system: "dnd5e".into(),
			path: "items/trinket.kdl".into(),
			version: Some("4b37d0e2a".into()),
			node_idx: 7,
		};
		assert_eq!(
			source.to_string(),
			"github://ghuser:homebrew@dnd5e/items/trinket.kdl?version=4b37d0e2a#7"
		);
	}
}
