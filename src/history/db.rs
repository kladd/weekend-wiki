use bincode::{Decode, Encode};

use crate::{history::delta::Delta, BINCODE_CONFIG};

/// page-name:1 = HistoryRecord { ... }
/// page-name:0 = HistoryRecord { ... }
/// page-name:VERSION = HistoryVersionRecord { 2 }

#[derive(Encode, Decode, Default)]
pub struct HistoryVersionRecord {
	next_version: u64,
}

pub struct HistoryKey(pub(super) String);

#[derive(Encode, Decode)]
pub struct HistoryRecord {
	pub(super) delta: Delta,
}

impl HistoryRecord {
	pub fn key(slug: &str, version: HistoryVersionRecord) -> String {
		format!("{slug}:{}", version.next_version)
	}

	pub fn new(slug: &str, old: &str, new: &str) -> Self {
		Self {
			delta: Delta::new(slug, old, new),
		}
	}
}

impl HistoryVersionRecord {
	pub fn key(slug: &str) -> String {
		format!("{slug}:VERSION")
	}

	pub fn next(&self) -> Self {
		Self {
			next_version: self.next_version + 1,
		}
	}
}

impl HistoryKey {
	pub fn revision(&self) -> String {
		self.0.split(':').last().unwrap().to_string()
	}
}

// TODO: These implementations are all identical.
impl HistoryVersionRecord {
	pub fn as_bytes(&self) -> Vec<u8> {
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}

	pub fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		let (me, _) =
			bincode::decode_from_slice(bytes.as_ref(), BINCODE_CONFIG).unwrap();
		me
	}
}

impl HistoryRecord {
	pub fn as_bytes(&self) -> Vec<u8> {
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}

	pub fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		let (me, _) =
			bincode::decode_from_slice(bytes.as_ref(), BINCODE_CONFIG).unwrap();
		me
	}
}
