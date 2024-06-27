fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	let modules_root_src = std::path::Path::new("./modules").to_owned();
	let modules_test_dst = std::path::Path::new("./tests/modules").to_owned();

	// Compile the list of local modules (path to module, the module id, and systems it supports)
	let mut modules = Vec::new();
	if let Ok(read_iter) = std::fs::read_dir(&modules_root_src) {
		for entry in read_iter {
			let Ok(entry) = entry else { continue };
			let Ok(metadata) = entry.metadata() else { continue };
			if !metadata.is_dir() {
				continue;
			}
			let module_id = entry.file_name().to_str().unwrap().to_owned();

			let mut system_ids = Vec::new();
			let Ok(read_iter_sys) = std::fs::read_dir(entry.path()) else {
				continue;
			};
			for entry in read_iter_sys {
				let Ok(entry) = entry else { continue };
				let Ok(metadata) = entry.metadata() else { continue };
				if !metadata.is_dir() {
					continue;
				}
				let system_id = entry.file_name().to_str().unwrap().to_owned();
				system_ids.push(system_id);
			}

			//log::debug!("Found module {module_id:?} with systems {system_ids:?}.");
			modules.push((entry.path(), module_id, system_ids));
		}
	}

	// Gather the list of file sources for each module
	let mut generated_module = Module::default();
	for (module_path, module_id, system_ids) in modules {
		for system_id in system_ids {
			//log::info!("Loading module \"{module_id}/{system_id}\"");
			let system_path = module_path.join(&system_id);
			for item in WalkDir::new(&system_path) {
				let Some(ext) = item.extension() else {
					continue;
				};
				if ext.to_str() != Some("kdl") {
					continue;
				}

				let item = item.display().to_string().replace("\\", "/");
				let item = std::path::Path::new(item.as_str()).to_owned();

				let Ok(relative_path) = item.strip_prefix(&modules_root_src) else {
					continue;
				};
				let relative_path = relative_path.with_extension("");

				let absolute_path = item.canonicalize().unwrap();
				let absolute_path = absolute_path.display().to_string().replace("\\", "/").replace("//?/", "");
				let absolute_path = std::path::Path::new(absolute_path.as_str()).to_owned();

				println!("cargo:rerun-if-changed={}", absolute_path.display());

				let entry = ModuleEntry {
					abs_path: absolute_path,
					rel_path: relative_path.clone(),
					module_id: module_id.clone(),
					system_id: system_id.clone(),
				};
				generated_module.insert(&relative_path, entry);
			}
		}
	}

	if modules_test_dst.exists() {
		std::fs::remove_dir_all(&modules_test_dst).unwrap();
	}
	generated_module.generate_module(&modules_test_dst);
}

struct WalkDir {
	iter: Option<std::fs::ReadDir>,
	stack: Vec<std::fs::ReadDir>,
}

impl WalkDir {
	fn new(path: impl AsRef<std::path::Path>) -> Self {
		Self { iter: std::fs::read_dir(path).ok(), stack: Vec::new() }
	}
}

impl Iterator for WalkDir {
	type Item = std::path::PathBuf;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(mut iter) = self.iter.take() else {
				return None;
			};
			let Some(item) = iter.next() else {
				// current entry has finished
				self.iter = self.stack.pop();
				continue;
			};
			let Ok(entry) = item else {
				self.iter = Some(iter);
				continue;
			};
			let Ok(metadata) = entry.metadata() else {
				self.iter = Some(iter);
				continue;
			};
			if metadata.is_dir() {
				let Ok(entry_iter) = std::fs::read_dir(entry.path()) else {
					self.iter = Some(iter);
					continue;
				};
				self.stack.push(iter);
				self.iter = Some(entry_iter);
				continue;
			}
			if !metadata.is_file() {
				self.iter = Some(iter);
				continue;
			}
			self.iter = Some(iter);
			return Some(entry.path());
		}
	}
}

#[derive(Debug)]
struct ModuleEntry {
	abs_path: std::path::PathBuf,
	rel_path: std::path::PathBuf,
	module_id: String,
	system_id: String,
}

#[derive(Default, Debug)]
struct Module {
	submodules: std::collections::BTreeMap<String, Module>,
	entries: std::collections::BTreeMap<String, ModuleEntry>,
	entry: Option<ModuleEntry>,
}
impl Module {
	fn make_str_symbol_safe(s: &String) -> String {
		use convert_case::{Case, Casing};
		let mut name = s.replace("-", "_");
		if name.starts_with(char::is_numeric) {
			name = format!("type_{name}");
		}
		if name.chars().all(char::is_alphabetic) {
			name = name.to_case(Case::Snake);
		}
		name
	}

	fn insert(&mut self, path: &std::path::Path, entry: ModuleEntry) {
		let mut parts = std::collections::VecDeque::with_capacity(path.components().count());
		for comp in path.components() {
			let Some(part) = comp.as_os_str().to_str() else { return };
			parts.push_back(part);
		}
		self.insert_pathvec(parts, entry);
	}

	fn insert_pathvec(&mut self, mut parts: std::collections::VecDeque<&str>, entry: ModuleEntry) {
		let Some(name) = parts.pop_front() else { return };

		if parts.is_empty() {
			if let Some(submodule) = self.submodules.get_mut(name) {
				submodule.entry = Some(entry);
			} else {
				self.entries.insert(name.to_owned(), entry);
			}
			return;
		}

		if !self.submodules.contains_key(name) {
			let mut module = Module::default();
			if let Some(module_content) = self.entries.remove(name) {
				module.entry = Some(module_content);
			}
			self.submodules.insert(name.to_owned(), module);
		}

		let Some(submodule) = self.submodules.get_mut(name) else {
			return;
		};
		submodule.insert_pathvec(parts, entry);
	}

	fn generate_module(&self, dir_path: &std::path::Path) {
		use std::io::Write;
		if !dir_path.exists() {
			std::fs::create_dir(dir_path).unwrap();
		}
		let module_path = match self.entry.is_some() {
			true => dir_path.with_extension("rs"),
			false => dir_path.join("mod.rs"),
		};
		let mut file = std::fs::File::create(module_path).unwrap();

		let mut lines = Vec::with_capacity(self.submodules.len() + self.entries.len());

		for (name, submodule) in &self.submodules {
			let module_name = Self::make_str_symbol_safe(name);
			submodule.generate_module(&dir_path.join(&module_name));
			lines.push(format!("mod {module_name};"));
		}

		for (name, src_path) in &self.entries {
			let module_name = Self::make_str_symbol_safe(name);
			let module_path = dir_path.join(&module_name).with_extension("rs");
			Self::generate_test(&module_path, &src_path);
			lines.push(format!("mod {module_name};"));
		}

		if let Some(entry) = &self.entry {
			lines.push("\n".to_owned());
			lines.push(Self::generate_test_content(entry));
		}

		let content = lines.join("\n");
		write!(file, "{content}").unwrap();
	}

	fn generate_test(module_path: &std::path::Path, entry: &ModuleEntry) {
		use std::io::Write;
		let mut file = std::fs::File::create(module_path).unwrap();
		write!(file, "{}", Self::generate_test_content(entry)).unwrap();
	}

	fn generate_test_content(entry: &ModuleEntry) -> String {
		let template = include_str!("./tests/modules_template");
		template
			.replace("{abs_path_to_kdl}", &entry.abs_path.display().to_string())
			.replace("{rel_path_to_kdl}", &entry.rel_path.display().to_string())
			.replace("{module_id}", &entry.module_id)
			.replace("{system_id}", &entry.system_id)
	}
}
