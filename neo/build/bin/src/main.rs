#[macro_use]
extern crate clap;
extern crate encage_build as build;
extern crate encage_build_schema as schema;
extern crate encage_build_dag as dag;

use clap::{App, AppSettings};
use std::fs::File;

fn main() {
	let app = App::new("encage-build");
	let app = clap_app! { @app (app)
		(author: "arcnmx")
		(about: "Encage build")
		(@arg ENGINE: --engine -e +takes_value "Build engine: runc, encage")
		(@arg FORMAT: --format -f +takes_value "Output format: makefile, ninja")
		(@arg OUTPUT: --output -o +takes_value "The output file")
		(@arg INPUT: +required "The build recipe")
	};

	let matches = app.get_matches();

	let input = matches.value_of("INPUT").unwrap();
	let engine = matches.value_of("ENGINE").unwrap_or("encage");
	let output = matches.value_of("OUTPUT").and_then(|o| matches.value_of("FORMAT").map(|f| (o, f)));

	let input = File::open(input).expect("failed to open input file");
	let schema = schema::load(input).expect("failed to read input file");
	let mut dag = dag::Dag::new();

	let mut last_stamp = None;
	for command in &schema.commands {
		let stamp = build::Stamp::new("stamp-", command);
		let command_context = build::CommandContext::new(&schema, command);
		let command_stamped = build::Stamper::new(command_context, stamp.clone());
		let stamp = dag.add_value(stamp);
		let node = dag.add_node(command_stamped);
		dag.add_output(node, stamp);
		if let Some(last_stamp) = last_stamp {
			dag.add_input(node, last_stamp);
		}
		last_stamp = Some(stamp);

		match *command {
			schema::Command::Copy(schema::CommandCopy { ref src, ref dest, .. }) => {
				/*let input = dag.add_value(src);
				let output = dag.add_value(schema.image.dest.join(dest));
				dag.add_input(node, input);
				dag.add_output(node, output);*/
			},
			_ => (),
		}
	}

	if let Some((output, format)) = output {
		assert_eq!(&format[..], "makefile");
		let out = File::create(output).expect("failed to create output");
		dag.write_makefile(out).expect("failed to write output");
	} else {
		unimplemented!()
	}
}
