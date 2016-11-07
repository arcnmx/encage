use serde_value::{Value, DeserializerError};
use plugins::{Plugin, ImageConfigurationContext};

pub struct FilesPlugin(());

impl FilesPlugin {
	pub fn new() -> Self {
		FilesPlugin(())
	}
}

impl Plugin for FilesPlugin {
	fn config_group(&self) -> Option<&str> {
		Some("base")
	}

	fn configure_image(&self, context: &mut ImageConfigurationContext) -> Result<(), DeserializerError> {
		if let Some(files) = context.user_data.remove("files") {
			// TODO
		}

		Ok(())
	}
}
