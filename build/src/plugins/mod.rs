use serde_value::DeserializerError;

use context::Context;
use config::Package;
use parse::{UserData, PluginData};

pub mod commands;
pub mod files;
pub mod handlebars;
pub mod build;
pub mod depends;

pub struct ImageConfigurationContext<'a> {
	pub root_package: &'a Package,
	pub package: &'a Package,
	pub user_data: &'a mut UserData,
	pub plugin_data: &'a mut PluginData,
}

pub struct ImageDependencyContext<'a> {
	pub context: &'a mut Context,
	pub package: &'a Package,
	pub plugin_data: &'a mut PluginData,
}

#[allow(unused_variables)]
pub trait Plugin {
	fn config_group(&self) -> Option<&str> { None }
	fn configure_image(&self, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> { Ok(()) }
	fn configure_image_dependencies(&self, context: &mut ImageDependencyContext) -> Result<(), ()> { Ok(()) }
}

pub struct Registry(Vec<Box<Plugin>>);

impl Registry {
	pub fn new() -> Self {
		Registry(Vec::new())
	}

	pub fn register_builtins(&mut self) {
		self.register_plugin(handlebars::HandlebarsPlugin::new());
		self.register_plugin(commands::CommandsPlugin::new());
		self.register_plugin(files::FilesPlugin::new());
		self.register_plugin(build::BuildPlugin::new());
		self.register_plugin(depends::DependsPlugin::new());
	}

	pub fn register_plugin<P: Plugin + 'static>(&mut self, p: P) {
		self.0.push(Box::new(p));
	}

	pub fn configure_image<S: AsRef<str>, I: IntoIterator<Item=S>>(&self, plugins: I, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> {
		let plugins = plugins.into_iter().map(|s| s.as_ref().to_owned()).collect::<Vec<_>>();

		for plugin in &self.0 {
			if let Some(config_group) = plugin.config_group() {
				if plugins.iter().any(|s| s == config_group) {
					try!(plugin.configure_image(&mut *context));
				}
			}
		}

		// TODO: warn on unknown plugins
		// return Err(::serde::de::Error::unknown_field(field)),

		Ok(())
	}

	pub fn configure_image_dependencies(&self, context: &mut ImageDependencyContext) -> Result<(), ()> {
		for plugin in &self.0 {
			try!(plugin.configure_image_dependencies(&mut *context));
		}

		let image_id = context.context.register_build_item(context.package, ::build::NullBuildItem::new("image", Some(::build::BUILD_CLASS_IMAGE.id())));
		let stage_id = context.context.register_build_item(context.package, ::build::NullBuildItem::new("stage", Some(::build::BUILD_CLASS_STAGE.id())));
		context.context.dependency_graph.link(image_id, stage_id);

		Ok(())
	}
}
