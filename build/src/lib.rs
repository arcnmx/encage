#![plugin(serde_macros)]
#![feature(plugin, custom_derive, custom_attribute, associated_consts)]

extern crate filesystem;
extern crate toml;
extern crate semver;
extern crate typemap;
extern crate serde;
extern crate serde_value;
extern crate unsafe_any;
extern crate url;
extern crate solvent;
extern crate snowflake;
extern crate hyper;

pub mod build;
pub mod config;
pub mod console;
pub mod context;
pub mod dependencies;
pub mod work;
pub mod util;
pub mod parse;
pub mod plugins;

pub use snowflake::ProcessUniqueId as Id;
pub use self::util::static_id::{StaticId, StaticIdDef};
