use std::path::PathBuf;
use config::Package;
use filesystem::FilesystemObject;

pub struct Workspace {
	root: PathBuf,
}

impl Workspace {
	pub fn staging_dir(&self, package: &Package) -> PathBuf {
		let mut root = self.root.clone();
		root.push(format!("{}-{}", package.name, package.version));
		root
	}

	pub fn staging_filesystem(&self, package: &Package) -> &FilesystemObject {
		unimplemented!()
	}
}
