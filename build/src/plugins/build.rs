use serde_value::{Value, DeserializerError};
use plugins::{Plugin, ImageConfigurationContext};

pub struct BuildPlugin(());

impl BuildPlugin {
	pub fn new() -> Self {
		BuildPlugin(())
	}
}

impl Plugin for BuildPlugin {
	fn config_group(&self) -> Option<&str> {
		Some("base")
	}

	fn configure_image(&self, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> {
		if let Some(build) = context.user_data.remove("build") {
			// TODO
		}

		if let Some(commands) = context.user_data.remove("build-commands") {
			// TODO
		}

		Ok(())
	}
}
