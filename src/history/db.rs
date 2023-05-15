use bincode::{Decode, Encode};

use crate::history::delta::Delta;

/// namespace/page-name/1       = HistoryRecord { ... }
/// namespace/page-name/0       = HistoryRecord { ... }
/// namespace/page-name/VERSION = HistoryVersionRecord { 2 }

#[derive(Encode, Decode, Default, Debug)]
pub struct HistoryVersionRecord {
	next_version: u64,
}

#[derive(Debug)]
pub struct HistoryKey(pub(super) String);

#[derive(Encode, Decode, Debug)]
pub struct HistoryRecord {
	pub(super) delta: Delta,
}

impl HistoryRecord {
	pub fn key(ns: &str, slug: &str, version: HistoryVersionRecord) -> String {
		format!("{ns}/{slug}/{}", version.next_version)
	}

	pub fn new(author: &str, old: &str, new: &str) -> Self {
		Self {
			delta: Delta::new(author, old, new),
		}
	}
}

impl HistoryVersionRecord {
	pub fn key(ns: &str, slug: &str) -> String {
		format!("{ns}/{slug}/VERSION")
	}

	pub fn next(&self) -> Self {
		Self {
			next_version: self.next_version + 1,
		}
	}
}

impl HistoryKey {
	pub fn revision(&self) -> String {
		self.0.split('/').last().unwrap().to_string()
	}

	pub fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		// TODO: Decoding errors.
		let key_str = String::from_utf8(bytes.as_ref().to_vec()).unwrap();
		// TODO: Components?
		Self(key_str)
	}
}
