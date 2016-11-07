use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
	Error,
	Note,
	Debug,
	Trace,
}

pub trait Console {
	fn log(&self, level: LogLevel, args: fmt::Arguments);
}

pub struct DefaultConsole;

impl Console for DefaultConsole {
	fn log(&self, level: LogLevel, args: fmt::Arguments) {
		println!("{:?}: {}", level, args);
	}
}
