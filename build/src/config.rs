use std::borrow::Cow;
use semver::{Version, VersionReq};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Package {
	pub name: String,
	pub version: Version,
}

#[derive(Debug, Clone)]
pub struct PackageQuery {
	pub name: String,
	pub version_req: VersionReq,
}

impl Package {
	pub fn absolute_name<'a, S: AsRef<str> + 'a>(&self, str: &'a S) -> Cow<'a, str> {
		let str = str.as_ref();
		if str.starts_with('.') {
			Cow::Owned(format!("{}{}", self.name, str))
		} else {
			Cow::Borrowed(str)
		}
	}

	pub fn to_query(&self) -> PackageQuery {
		PackageQuery {
			name: self.name.clone(),
			version_req: VersionReq::exact(&self.version),
		}
	}

	pub fn into_absolute<S: AsRef<str>>(self, str: S) -> Self {
		Package {
			name: self.absolute_name(&str).into_owned(),
			version: self.version,
		}
	}
}
