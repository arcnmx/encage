use ::{StaticIdDef, StaticId, Id};
use config::Package;
use work::Workspace;
use std::fmt;
use filesystem::FilesystemObject;

pub type BuildClass = StaticId;

pub trait BuildItem: fmt::Display + fmt::Debug {
	fn class(&self) -> Option<BuildClass> { None }

	fn register_dependencies(&self, context: &mut BuildDependencyContext) -> Result<(), ()>;
	fn build(&self, context: &mut BuildContext) -> Result<(), ()>;
}

pub struct BuildContext<'a> {
	pub package: &'a Package,
	pub workspace: &'a Workspace,
	pub source_filesystem: &'a mut FilesystemObject,
}

pub struct BuildDependencyContext<'a> {
	pub package: &'a Package,
	pub depends_on: &'a mut FnMut(&Package, BuildClass),
	pub required_by: &'a mut FnMut(&Package, BuildClass),
}

impl<'a> BuildDependencyContext<'a> {
	pub fn depends_on(&mut self, p: &Package, b: BuildClass) {
		(self.depends_on)(p, b);
	}

	pub fn required_by(&mut self, p: &Package, b: BuildClass) {
		(self.required_by)(p, b);
	}
}

pub struct BuildItems {
	build_items: Vec<(Id, Package, Box<BuildItem>)>,
}

impl BuildItems {
	pub fn new() -> Self {
		BuildItems {
			build_items: Vec::new(),
		}
	}

	pub fn register_build_item(&mut self, id: Id, package: &Package, b: Box<BuildItem>) {
		self.build_items.push((id, package.clone(), b));
	}

	pub fn dependencies_matching(&self, package: &Package, class: BuildClass) -> Vec<(Id, &BuildItem)> {
		self.build_items.iter().filter_map(|&(id, ref build_package, ref item)| if build_package == package && item.class() == Some(class) {
			Some((id, &**item))
		} else {
			None
		}).collect()
	}

	pub fn iter(&self) -> ::std::slice::Iter<(Id, Package, Box<BuildItem>)> {
		self.build_items.iter()
	}

	pub fn get(&self, id: Id) -> Option<(&Package, &BuildItem)> {
		self.build_items.iter().filter_map(|&(build_id, ref package, ref item)| if id == build_id {
			Some((package, &**item))
		} else {
			None
		}).next()
	}
}

pub static BUILD_CLASS_PRESTAGE: StaticIdDef = StaticIdDef::INIT;
pub static BUILD_CLASS_STAGE: StaticIdDef = StaticIdDef::INIT;
pub static BUILD_CLASS_IMAGE: StaticIdDef = StaticIdDef::INIT;

#[derive(Debug)]
pub struct NullBuildItem(String, Option<BuildClass>);

impl NullBuildItem {
	pub fn new<S: Into<String>>(s: S, class: Option<BuildClass>) -> Self {
		NullBuildItem(s.into(), class)
	}
}

impl BuildItem for NullBuildItem {
	fn class(&self) -> Option<BuildClass> { self.1 }

	fn register_dependencies(&self, _context: &mut BuildDependencyContext) -> Result<(), ()> { Ok(()) }
	fn build(&self, _context: &mut BuildContext) -> Result<(), ()> { Ok(()) }
}

impl fmt::Display for NullBuildItem {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}
