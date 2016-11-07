use std::io::{self, Write};
use std::collections::BTreeMap;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct DagValueId(u64);
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct DagNodeId(u64);

pub struct Dag<T, I> {
	count: u64,
	nodes: BTreeMap<DagNodeId, (T, Vec<DagValueId>, Vec<DagValueId>)>,
	values: BTreeMap<DagValueId, I>,
}

impl<T, I> Dag<T, I> {
	pub fn new() -> Self {
		Dag {
			count: 0,
			nodes: BTreeMap::new(),
			values: BTreeMap::new(),
		}
	}

	fn node_id(&mut self) -> DagNodeId {
		let id = DagNodeId(self.count);
		self.count += 1;
		id
	}

	fn value_id(&mut self) -> DagValueId {
		let id = DagValueId(self.count);
		self.count += 1;
		id
	}

	pub fn add_value(&mut self, v: I) -> DagValueId {
		let id = self.value_id();
		self.values.insert(id, v);
		id
	}

	pub fn add_node(&mut self, v: T) -> DagNodeId {
		let id = self.node_id();
		self.nodes.insert(id, (v, Vec::new(), Vec::new()));
		id
	}

	pub fn add_input(&mut self, node: DagNodeId, input: DagValueId) {
		self.nodes.get_mut(&node).unwrap().1.push(input);
	}

	pub fn add_output(&mut self, node: DagNodeId, output: DagValueId) {
		self.nodes.get_mut(&node).unwrap().2.push(output);
	}
}

impl<T: ToShellString, I: ToFilePath> Dag<T, I> {
	pub fn write_makefile<W: Write>(&self, mut w: W) -> io::Result<()> {
		for (_, &(ref node, ref inputs, ref outputs)) in &self.nodes {
			for output in outputs {
				let output = self.values.get(output).unwrap();
				try!(write!(w, "{} ", output.to_file_path()));
			}
			try!(write!(w, ": "));
			for input in inputs {
				let input = self.values.get(input).unwrap();
				try!(write!(w, "{} ", input.to_file_path()));
			}
			try!(writeln!(w, ""));
			try!(write!(w, "\t"));
			try!(write!(w, "{}", node.to_shell_string()));
			try!(writeln!(w, ""));
		}

		Ok(())
	}
}

pub trait ToShellString {
	fn to_shell_string(&self) -> String;
}

pub trait ToFilePath {
	fn to_file_path(&self) -> String;
}
