use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleId {
	File(PathBuf),
	Github {
		user_org: String,
		repository: String,
	},
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceId {
	pub module: ModuleId,
	pub path: PathBuf,
	pub version: Option<String>,
	pub node_idx: usize,
}

impl ToString for SourceId {
	fn to_string(&self) -> String {
		let mut url_str = match &self.module {
			ModuleId::File(base_path) => {
				format!("file://{}", base_path.join(&self.path).display())
			}
			ModuleId::Github {
				user_org,
				repository,
			} => {
				format!(
					"url://{user_org}:{repository}@github/{}",
					self.path.display()
				)
			}
		}
		.replace("\\", "/");
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
		let module = match (url.scheme(), url.host_str()) {
			("file", _) => ModuleId::File(PathBuf::new()),
			("url", Some("github")) => ModuleId::Github {
				user_org: url.username().to_owned(),
				repository: url
					.password()
					.ok_or(SourceIdParseError::MissingRepository)?
					.to_string(),
			},
			(scheme, host) => {
				return Err(SourceIdParseError::UnrecognizedModule(
					scheme.to_owned(),
					host.map(str::to_owned),
				)
				.into());
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
			module,
			path,
			version,
			node_idx,
		})
	}
}

impl SourceId {
	pub fn relative_to(mut self, module: &ModuleId) -> anyhow::Result<Self> {
		match (module, &mut self.module, &mut self.path) {
			(ModuleId::File(base), ModuleId::File(derived), path) => {
				*path = derived.join(&path).strip_prefix(&base)?.to_owned();
				*derived = base.clone();
			}
			_ => {}
		}
		Ok(self)
	}
}

#[derive(thiserror::Error, Debug)]
enum SourceIdParseError {
	#[error("Missing repository name in password field of url")]
	MissingRepository,
	#[error("Unrecognized module service {0}:{1:?}")]
	UnrecognizedModule(String, Option<String>),
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn file_no_module() -> anyhow::Result<()> {
		let src = "file:///c/Users/thisuser/Desktop/homebrew/items/trinket.kdl#32";
		let source_id = SourceId::from_str(src)?;
		assert_eq!(
			source_id,
			SourceId {
				module: ModuleId::File(PathBuf::new()),
				path: "/c/Users/thisuser/Desktop/homebrew/items/trinket.kdl".into(),
				version: None,
				node_idx: 32,
			}
		);
		Ok(())
	}

	#[test]
	fn file_relative_to() -> anyhow::Result<()> {
		let module = ModuleId::File("/c/Users/thisuser/Desktop/homebrew".into());
		let source = SourceId {
			module: ModuleId::File(PathBuf::new()),
			path: "/c/Users/thisuser/Desktop/homebrew/items/trinket.kdl".into(),
			version: None,
			node_idx: 0,
		};
		assert_eq!(
			source.relative_to(&module)?,
			SourceId {
				module,
				path: "items/trinket.kdl".into(),
				version: None,
				node_idx: 0,
			}
		);
		Ok(())
	}

	#[test]
	fn file_to_str() {
		let source = SourceId {
			module: ModuleId::File("/c/Users/thisuser/Desktop/homebrew".into()),
			path: "items/trinket.kdl".into(),
			version: None,
			node_idx: 0,
		};
		assert_eq!(
			source.to_string(),
			"file:///c/Users/thisuser/Desktop/homebrew/items/trinket.kdl"
		);
	}

	#[test]
	fn github() -> anyhow::Result<()> {
		let src = "url://ghuser:homebrew@github/items/trinket.kdl?version=4b37d0e2a#5";
		let source_id = SourceId::from_str(src)?;
		assert_eq!(
			source_id,
			SourceId {
				module: ModuleId::Github {
					user_org: "ghuser".into(),
					repository: "homebrew".into()
				},
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
			path: "items/trinket.kdl".into(),
			version: Some("4b37d0e2a".into()),
			node_idx: 7,
		};
		assert_eq!(
			source.to_string(),
			"url://ghuser:homebrew@github/items/trinket.kdl?version=4b37d0e2a#7"
		);
	}
}
