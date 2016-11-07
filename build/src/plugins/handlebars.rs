extern crate handlebars;

use std::collections::BTreeMap;
use serde_value::{Value, DeserializerError};
use typemap::Key;
use self::handlebars::{Handlebars, TemplateError};
use plugins::{Plugin, ImageConfigurationContext};

pub struct HandlebarsPlugin(());

impl HandlebarsPlugin {
	pub fn new() -> Self {
		HandlebarsPlugin(())
	}
}

impl Plugin for HandlebarsPlugin {
	fn config_group(&self) -> Option<&str> {
		Some("handlebars")
	}

	fn configure_image(&self, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> {
		if let Some(vars) = context.user_data.remove("vars") {
			context.plugin_data.set::<Self>(Vars {
				inner: try!(vars.deserialize_into()),
			});
		}

		Ok(())
	}
}

impl HandlebarsPlugin {
	pub fn transform_string(str: String, context: &ImageConfigurationContext) -> Result<String, DeserializerError> {
		let mut handlebars = Handlebars::new();
		try!(handlebars.register_template_string("main", str)
			.map_err(|e| match e {
				TemplateError::UnclosedBraces(line, col) => DeserializerError::Syntax(format!("Unclosed brace at {}:{}", line, col)),
				TemplateError::UnexpectedClosingBraces(line, col) => DeserializerError::Syntax(format!("Unexpected closing brace at {}:{}", line, col)),
				TemplateError::MismatchingClosedHelper(line, col, ref expected, ref actual) => DeserializerError::Syntax(format!("Mismatched closing helper {} (expected {}) at {}:{}", actual, expected, line, col)),
				TemplateError::UnclosedHelper(line, col, ref tag) => DeserializerError::Syntax(format!("Unclosed helper {} at {}:{}", tag, line, col)),
			})
		);

		#[derive(Debug, Serialize)]
		struct Data<'a> {
			#[serde(skip_serializing_if_none)]
			vars: Option<&'a BTreeMap<String, Value>>,
			package: &'a Package,
		}

		let data = Data {
			vars: context.plugin_data.get::<Self>().map(|v| &v.inner),
		};

		handlebars.render("main", &data).map_err(|_| unimplemented!())
	}
}

impl Key for HandlebarsPlugin {
	type Value = Vars;
}

#[derive(Clone, Debug, Default)]
pub struct Vars {
	inner: BTreeMap<String, Value>,
}
