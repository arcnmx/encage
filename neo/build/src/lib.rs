extern crate encage_build_schema as schema;
extern crate encage_build_dag as dag;

use std::hash::{Hash, Hasher, SipHasher};
use std::path::Path;
use std::borrow::Cow;
use std::fmt;

#[derive(Clone, Hash)]
pub struct Stamp {
	prefix: String,
	hash: u64,
}

impl Stamp {
	pub fn new<S: Into<String>, H: Hash>(prefix: S, hash: &H) -> Self {
		let mut s = SipHasher::new();
		hash.hash(&mut s);

		Stamp {
			prefix: prefix.into(),
			hash: s.finish(),
		}
	}
}

impl fmt::Display for Stamp {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}{:016x}.stamp", self.prefix, self.hash)
	}
}

impl dag::ToFilePath for Stamp {
	fn to_file_path(&self) -> String {
		format!("{}", self)
	}
}

pub struct CommandContext<'a> {
	image: &'a schema::ImageRecipe,
	command: &'a schema::Command,
}

impl<'a> CommandContext<'a> {
	pub fn new(image: &'a schema::ImageRecipe, command: &'a schema::Command) -> Self {
		CommandContext {
			image: image,
			command: command,
		}
	}
}

impl<'a> dag::ToShellString for CommandContext<'a> {
	fn to_shell_string(&self) -> String {
		match *self.command {
			schema::Command::Copy(ref copy) => {
				let dest = Path::new(&self.image.image.dest).join(rootless(&copy.dest));
				let dest = dest.display().to_string();
				let mode = if let Some(mode) = copy.mode {
					Cow::Owned(format!("{:04o}", mode))
				} else {
					Cow::Borrowed("0644")
				};
				shell_string(&["install", "-Dm", &mode[..], &copy.src[..], &dest[..]])
			},
			schema::Command::Exec(ref exec) => {
				use std::iter::once;

				exec.commands.iter().map(|command| {
					let args: Vec<&str> = match *command {
						schema::CommandArgs::Shell(ref s) => ["sh", "-ec", &s[..]].iter().map(|&s| s).collect(),
						schema::CommandArgs::Exec { ref process, ref args } => once(&process[..]).chain(args.iter().map(|s| &s[..])).collect(),
					};

					match exec.kind {
						schema::CommandExecType::Ocf { ref root } => {
							shell_string(["encage-run", "ocf", &root[..]].iter().map(|&s| s).chain(args))
						},
						schema::CommandExecType::Image => {
							shell_string(["encage-run", "exec", &self.image.image.dest[..]].iter().map(|&s| s).chain(args))
						},
						schema::CommandExecType::Host => {
							shell_string(args)
						},
					}
				}).fold(String::new(), |s, c| if s.len() == 0 { s } else { s + " && " } + &c)
			},
		}
	}
}

pub struct Stamper<T> {
	inner: T,
	stamp: Stamp,
}

impl<T> Stamper<T> {
	pub fn new(inner: T, stamp: Stamp) -> Self {
		Stamper {
			inner: inner,
			stamp: stamp,
		}
	}
}

impl<T: dag::ToShellString> dag::ToShellString for Stamper<T> {
	fn to_shell_string(&self) -> String {
		format!("{} && {}", self.inner.to_shell_string(), shell_string(&["touch", &self.stamp.to_string()[..]]))
	}
}

fn shell_string<S: AsRef<str>, I: IntoIterator<Item=S>>(args: I) -> String {
	let mut out = String::new();

	for s in args.into_iter() {
		let s = s.as_ref();

		if out.len() > 0 {
			out.push(' ');
		}

		// http://stackoverflow.com/questions/15783701/which-characters-need-to-be-escaped-in-bash-how-do-we-know-it/20053121#20053121
		let safe = |c| ",._+:@%/-".contains(c) || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9');
		if s.len() == 0 {
			out.push_str("''");
		} else if s.contains(|c| !safe(c)) {
			let s = s.split('\'').map(|v| format!("'{}'", v)).fold(String::new(),
				|s, v| s + &v[..] + "\\'");
			out.push_str(&s[..s.len() - 2]);
		} else {
			out.push_str(&s[..]);
		}
	}

	out
}

fn rootless<'a, S: AsRef<str> + 'a>(s: &'a S) -> &'a str {
	let mut s = s.as_ref();

	while s.starts_with('/') {
		s = &s[1..];
	}

	s
}
