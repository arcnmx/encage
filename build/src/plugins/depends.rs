use serde_value::{Value, DeserializerError};
use plugins::{Plugin, ImageConfigurationContext};

pub struct DependsPlugin(());

impl DependsPlugin {
	pub fn new() -> Self {
		DependsPlugin(())
	}
}

impl Plugin for DependsPlugin {
	fn config_group(&self) -> Option<&str> {
		Some("base")
	}

	fn configure_image(&self, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> {
		if let Some(depends) = context.user_data.remove("depends") {
			// TODO
		}

		Ok(())
	}
}
