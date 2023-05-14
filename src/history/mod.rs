use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};
use bincode::{Decode, Encode};
use rocksdb::IteratorMode;

use crate::{
	document::Document, history::diff::Delta, not_found, Context,
	BINCODE_CONFIG, HIST_CF, PAGE_CF,
};

pub mod diff;

#[derive(Encode, Decode, Debug)]
pub struct HistoryRecord {
	delta: Delta,
}

#[derive(Encode, Decode, Default)]
pub struct HistoryVersion(pub u64);

pub struct HistoryResult {
	diff: String,
	version: String,
}

impl HistoryVersion {
	pub fn next(&self) -> Self {
		Self(self.0 + 1)
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		// TODO: This is like the same for everything. Default trait impl it or
		// something.
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}
}

impl HistoryRecord {
	pub fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		let (record, _) =
			bincode::decode_from_slice(bytes.as_ref(), BINCODE_CONFIG).unwrap();
		record
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}
}

#[derive(Template)]
#[template(path = "history.html")]
pub struct HistoryResponse {
	title: String,
	slug: String,
	versions: Vec<HistoryResult>,
}

#[axum_macros::debug_handler]
pub async fn get(
	Path(slug): Path<String>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let doc_maybe = db.get_cf(db.cf_handle(PAGE_CF).unwrap(), &slug).unwrap();
	let page = match doc_maybe {
		Some(bytes) => Document::from_bytes(bytes),
		_ => return not_found().await.into_response(),
	};

	let iter = db.prefix_iterator_cf(db.cf_handle(HIST_CF).unwrap(), &slug);
	let mut versions = iter
		.map(Result::unwrap)
		.filter_map(|(k, v)| {
			let version = dbg!(String::from_utf8(k.to_vec()).unwrap());
			if version.contains("VERSION")
				|| !version.starts_with(&format!("{}:", &slug))
			{
				None
			} else {
				Some((version, HistoryRecord::from_bytes(v)))
			}
		})
		.map(|(version, record)| HistoryResult {
			diff: record.delta.patch,
			version: version.split(":").last().unwrap().to_string(),
		})
		.collect::<Vec<_>>();

	// TODO: HACK.
	versions.reverse();

	Html(
		HistoryResponse {
			title: page.title().clone(),
			slug,
			versions,
		}
		.render()
		.unwrap(),
	)
	.into_response()
}

impl From<Vec<u8>> for HistoryVersion {
	fn from(value: Vec<u8>) -> Self {
		let (version, _) =
			bincode::decode_from_slice(value.as_slice(), BINCODE_CONFIG)
				.unwrap();
		version
	}
}

impl HistoryRecord {
	pub fn new(slug: &str, prev: &str, new: &str) -> Self {
		Self {
			delta: Delta::new(slug, prev, new),
		}
	}
}
