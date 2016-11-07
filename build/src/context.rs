use Id;
use std::fmt;
use config::{Package, PackageQuery};
use console::Console;
use dependencies::DependencyGraph;
use build::{self, BuildItems, BuildItem};

pub struct Context {
	pub console: Box<Console>,
	pub packages: Vec<Package>,
	pub dependency_graph: DependencyGraph,
	pub build_items: BuildItems,
}

impl Context {
	pub fn new<C: Console + 'static>(console: C) -> Self {
		Context {
			console: Box::new(console),
			packages: Vec::new(),
			dependency_graph: DependencyGraph::new(),
			build_items: BuildItems::new(),
		}
	}

	pub fn query_package<'a>(&'a self, query: &PackageQuery) -> Option<&'a Package> {
		self.packages.iter().find(|package| package.name == query.name && query.version_req.matches(&package.version))
	}

	pub fn register_build_item<B: BuildItem + 'static>(&mut self, package: &Package, b: B) -> Id {
		let id = self.dependency_graph.register();
		self.build_items.register_build_item(id, package, Box::new(b));
		id
	}

	pub fn resolve_dependencies(&mut self) -> Result<(), ()> {
		use std::cell::RefCell;

		let deps = RefCell::new(&mut self.dependency_graph);
		let build_items = &self.build_items;
		for item in build_items.iter() {
			let id = item.0;
			let package = &item.1;
			let item = &item.2;

			let mut context = build::BuildDependencyContext {
				depends_on: &mut |p, c| {
					let mut deps = deps.borrow_mut();
					for (dep_id, _) in build_items.dependencies_matching(p, c) {
						deps.link(id, dep_id);
					}
				},
				required_by: &mut |p, c| {
					let mut deps = deps.borrow_mut();
					for (dep_id, _) in build_items.dependencies_matching(p, c) {
						deps.link(dep_id, id);
					}
				},
				package: package,
			};

			try!(item.register_dependencies(&mut context));
		}

		Ok(())
	}

	pub fn image_dependency(&self, package: &Package) -> Option<Id> {
		self.build_items.dependencies_matching(package, build::BUILD_CLASS_IMAGE.id()).into_iter().next().map(|(id, _)| id)
	}
}

impl fmt::Debug for Context {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		unimplemented!()
	}
}
