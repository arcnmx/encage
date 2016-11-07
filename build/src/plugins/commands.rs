use serde_value::{Value, DeserializerError};
use plugins::{Plugin, ImageConfigurationContext};

pub struct CommandsPlugin(());

impl CommandsPlugin {
	pub fn new() -> Self {
		CommandsPlugin(())
	}
}

impl Plugin for CommandsPlugin {
	fn config_group(&self) -> Option<&str> {
		Some("base")
	}

	fn configure_image(&self, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> {
		if let Some(commands) = context.user_data.remove("commands") {
			// TODO
		}

		Ok(())
	}
}
