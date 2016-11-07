use std::collections::BTreeMap;
use typemap::{self, DebugMap};
use semver::Version;
use serde::{de, Serialize, Serializer, Deserialize, Deserializer};
use serde_value::{self, Value, DeserializerError};
use std::error::Error;
use std::{io, fmt};
use plugins::{Registry, ImageConfigurationContext};
use config::Package;

#[derive(Debug)]
pub struct PluginData(DebugMap);

impl PluginData {
	pub fn new() -> Self {
		PluginData(DebugMap::custom())
	}

	pub fn set<K: typemap::Key>(&mut self, v: K::Value) where K::Value: fmt::Debug {
		self.0.insert::<K>(v);
	}

	pub fn get<K: typemap::Key>(&self) -> Option<&K::Value> where K::Value: fmt::Debug {
		self.0.get::<K>()
	}

	pub fn get_or_default<K: typemap::Key>(&mut self) -> &mut K::Value where K::Value: fmt::Debug + Default {
		self.0.entry::<K>().or_insert_with(Default::default)
	}
}

pub type UserData = BTreeMap<String, Value>;

#[derive(Debug)]
pub struct ImageDesc {
	pub package: Package,
	pub kind: ImageKind,
	pub plugin_data: PluginData,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageKind {
	Host,
	Container,
}

#[derive(Deserialize)]
struct SerializePackage {
	name: String,
	version: SerializeVersion,
	plugins: Vec<String>,
}

#[derive(Deserialize)]
struct SerializeRoot {
	package: SerializePackage,
	image: Vec<SerializeImageDesc>,
}

struct SerializeImageDesc {
	name: String,
	kind: ImageKind,
	user_data: BTreeMap<String, Value>,
}

impl Deserialize for SerializeImageDesc {
	fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		struct Data {
			name: String,
			#[serde(default, rename="type")]
			kind: ImageKind,
		}

		Value::deserialize(d).and_then(|value| {
			let mut de = serde_value::Deserializer::new(value);
			let image = try!(Data::deserialize(&mut de).map_err(DeserializerError::into_error));
			Ok(SerializeImageDesc {
				name: image.name,
				kind: image.kind,
				user_data: try!(Deserialize::deserialize(&mut de).map_err(DeserializerError::into_error)),
			})
		})
	}
}

impl Default for ImageKind {
	fn default() -> Self {
		ImageKind::Container
	}
}

impl Deserialize for ImageKind {
	fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
		struct V;

		impl de::Visitor for V {
			type Value = ImageKind;

			fn visit_str<E: de::Error>(&mut self, value: &str) -> Result<Self::Value, E> {
				ImageKind::from_str(value).ok_or(E::syntax("unknown image type"))
			}
		}

		d.visit(V)
	}
}

impl ImageKind {
	pub fn from_str<S: AsRef<str>>(str: S) -> Option<Self> {
		Some(match str.as_ref() {
			"host" => ImageKind::Host,
			"container" => ImageKind::Container,
			_ => return None,
		})
	}
}

struct SerializeVersion(Version);

impl Deserialize for SerializeVersion {
	fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
		struct V;

		impl de::Visitor for V {
			type Value = SerializeVersion;

			fn visit_str<E: de::Error>(&mut self, value: &str) -> Result<Self::Value, E> {
				Version::parse(value).map(SerializeVersion).map_err(|e| E::syntax(e.description()))
			}
		}

		d.visit(V)
	}
}

impl Serialize for SerializeVersion {
	fn serialize<S: Serializer>(&self, s: &mut S) -> Result<(), S::Error> {
		s.visit_str(&self.0.to_string())
	}
}

pub fn parse<R: io::Read>(r: &mut R, plugin_registry: &Registry) -> io::Result<Vec<ImageDesc>> {
	use toml::{self, Decoder, Parser};
	fn err<E: Into<Box<Error + Send + Sync>>>(e: E) -> io::Error {
		io::Error::new(io::ErrorKind::InvalidData, e)
	}

	let mut str = String::new();
	try!(r.read_to_string(&mut str));
	str = str.replace("\t", "    ");

	let mut parser = Parser::new(&str);
	let table = parser.parse();
	if !parser.errors.is_empty() {
		return Err(err(parser.errors.into_iter().next().expect("unreachable")))
	}

	if let Some(table) = table {
		let root = try!(SerializeRoot::deserialize(&mut Decoder::new(toml::Value::Table(table))).map_err(err));

		let root_package = Package {
			name: root.package.name,
			version: root.package.version.0,
		};

		let plugins = root.package.plugins;
		let plugins = plugins.iter().collect::<Vec<_>>();

		let images = try!(root.image.into_iter().map(|i| {
			let package = root_package.clone().into_absolute(i.name);
			let kind = i.kind;
			let mut user_data = i.user_data;
			let mut plugin_data = PluginData::new();
			plugin_registry.configure_image(&plugins, &mut ImageConfigurationContext {
				user_data: &mut user_data,
				plugin_data: &mut plugin_data,
				package: &package,
				root_package: &root_package,
			}).and_then(|_| if let Some((key, _)) = user_data.iter().next() {
				Err(de::Error::unknown_field(key))
			} else {
				Ok(())
			}).map(move |_| ImageDesc {
				package: package,
				kind: kind,
				plugin_data: plugin_data,
			})
		}).collect::<Result<_, _>>().map_err(err));

		Ok(images)
	} else {
		Err(io::Error::new(io::ErrorKind::InvalidData, "empty input"))
	}
}
