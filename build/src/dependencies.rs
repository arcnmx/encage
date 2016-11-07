use solvent::{DepGraph, DepGraphIterator};
use Id;

pub struct DependencyGraph(DepGraph<Id>);

impl DependencyGraph {
	pub fn new() -> Self {
		DependencyGraph(DepGraph::new())
	}

	pub fn register(&mut self) -> Id {
		let id = Id::new();
		self.0.register_dependencies(id, &[]);
		id
	}

	pub fn link(&mut self, id: Id, dep: Id) {
		self.0.register_dependency(id, dep)
	}

	pub fn walk(&mut self, id: Id) -> DepGraphIterator<Id> {
		self.0.dependencies_of(id)
	}
}
