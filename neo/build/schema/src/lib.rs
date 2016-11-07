extern crate toml;
extern crate serde;
extern crate serde_value;

use serde_value::{Value, DeserializerError};
use serde::de::Error;
use std::collections::BTreeMap;
use std::io::{self, Read};

pub type StringMap = BTreeMap<String, Value>;

#[derive(Clone, Debug, Hash, Deserialize)]
pub struct ImageRecipe {
	pub image: Image,
	#[serde(rename = "command")]
	pub commands: Vec<Command>,
	#[serde(rename = "mount")]
	pub mounts: Vec<Mount>,
}

#[derive(Clone, Debug, Hash, Deserialize)]
pub struct Image {
	pub dest: String,
}

#[derive(Clone, Debug, Hash, Deserialize)]
pub enum Mount {
	Bind(MountBind),
}

#[derive(Clone, Debug, Hash, Deserialize)]
pub struct MountBind {
	pub name: String,
	pub src: String,
}

#[derive(Clone, Debug, Hash)]
pub enum Command {
	Copy(CommandCopy),
	Exec(CommandExec),
}

#[derive(Clone, Debug, Hash, Deserialize)]
pub struct CommandCopy {
	pub src: String,
	pub dest: String,
	#[serde(default, deserialize_with = "deserialize_octal")]
	pub mode: Option<u32>,
}

#[derive(Clone, Debug, Hash)]
pub struct CommandExec {
	pub kind: CommandExecType,
	pub cwd: Option<String>,
	pub commands: Vec<CommandArgs>,
}

#[derive(Clone, Debug, Hash)]
pub enum CommandArgs {
	Shell(String),
	Exec {
		process: String,
		args: Vec<String>,
	},
}

#[derive(Clone, Debug, Hash, Deserialize)]
pub enum CommandExecType {
	Image,
	Host,
	Ocf {
		root: String,
	},
}

fn deserialize_octal<D: serde::Deserializer>(d: &mut D) -> Result<Option<u32>, D::Error> {
	<String as serde::Deserialize>::deserialize(d)
		.and_then(|v| u32::from_str_radix(&v, 8)
			.map_err(|e| D::Error::invalid_value("mode must be octal"))
		).map(Some)
}

impl serde::Deserialize for CommandArgs {
	fn deserialize<D: serde::Deserializer>(d: &mut D) -> Result<Self, D::Error> {
		Value::deserialize(d).and_then(|v| match v {
			Value::String(str) => Ok(CommandArgs::Shell(str)),
			v => v.deserialize_into::<Vec<String>>().map_err(DeserializerError::into_error).and_then(|args| if args.len() >= 1 {
				let mut args = args.into_iter();
				Ok(CommandArgs::Exec {
					process: args.next().unwrap(),
					args: args.collect(),
				})
			} else {
				Err(D::Error::invalid_value("at least one argument is required"))
			}),
		})
	}
}

impl serde::Deserialize for Command {
	fn deserialize<D: serde::Deserializer>(d: &mut D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		struct Exec {
			#[serde(default)]
			cwd: Option<String>,
			#[serde(default)]
			root: Option<String>,
			#[serde(default)]
			command: Option<CommandArgs>,
			#[serde(default)]
			commands: Vec<CommandArgs>,
		}

		StringMap::deserialize(d).and_then(|mut v| {
			let kind = try!(v.remove("type").ok_or_else(|| D::Error::invalid_value("expected type field")));
			let kind: String = try!(kind.deserialize_into().map_err(DeserializerError::into_error));
			let v = Value::Map(v.into_iter().map(|(k, v)| (Value::String(k), v)).collect());
			match &kind[..] {
				"copy" => v.deserialize_into::<CommandCopy>().map_err(DeserializerError::into_error).map(Command::Copy),
				"host" | "image" | "ocf" => v.deserialize_into::<Exec>().map_err(DeserializerError::into_error)
					.and_then(|v| Ok(Command::Exec(CommandExec {
						kind: match &kind[..] {
							"host" => CommandExecType::Host,
							"image" => CommandExecType::Image,
							"ocf" => CommandExecType::Ocf {
								root: try!(v.root.ok_or_else(|| D::Error::invalid_value("ocf requires root"))),
							},
							_ => unreachable!(),
						},
						cwd: v.cwd,
						commands: {
							let commands: Vec<_> = v.command.into_iter().chain(v.commands.into_iter()).collect();
							if commands.len() == 0 {
								return Err(D::Error::missing_field("command"))
							} else {
								commands
							}
						},
					}))
				),
				_ => Err(D::Error::invalid_value("unknown command type")),
			}
		})
	}
}

pub fn load<R: Read>(mut r: R) -> io::Result<ImageRecipe> {
	let mut s = Vec::new();
	try!(r.read_to_end(&mut s));
	let s = try!(String::from_utf8(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)));
	let mut parser = toml::Parser::new(&s);
	let v = try!(parser.parse().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, parser.errors.remove(0))));
	serde::Deserialize::deserialize(&mut toml::Decoder::new(toml::Value::Table(v)))
		.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
