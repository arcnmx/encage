extern crate encage_build;

#[test]
fn parse() {
	let data = include_bytes!("sample.toml");

	let mut plugins = encage_build::plugins::Registry::new();
	plugins.register_builtins();
	let config = encage_build::parse::parse(&mut &data[..], &plugins);

	config.expect("parse failed");
}
