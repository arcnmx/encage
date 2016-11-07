#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct StaticId(*const StaticIdDef);
unsafe impl Sync for StaticId { }
unsafe impl Send for StaticId { }

#[derive(Debug)]
pub struct StaticIdDef(u8);

impl StaticIdDef {
	pub const INIT: StaticIdDef = StaticIdDef(0);

	pub fn id(&'static self) -> StaticId {
		self.into()
	}
}

impl From<&'static StaticIdDef> for StaticId {
	fn from(this: &'static StaticIdDef) -> Self {
		StaticId(this as *const _)
	}
}
