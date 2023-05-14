use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};

use crate::{
	document::Document,
	history::{
		db::{HistoryKey, HistoryRecord},
		view::{HistoryRevisionView, HistoryView},
	},
	not_found, Context, HIST_CF, PAGE_CF,
};

pub mod db;
mod delta;
pub mod view;

#[axum_macros::debug_handler]
pub async fn get(
	Path(slug): Path<String>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	// Check that the document exists.
	let doc_maybe = db.get_cf(db.cf_handle(PAGE_CF).unwrap(), &slug).unwrap();
	let page = match doc_maybe {
		Some(bytes) => Document::from_bytes(bytes),
		_ => return not_found().await.into_response(),
	};

	let iter = db.prefix_iterator_cf(db.cf_handle(HIST_CF).unwrap(), &slug);
	let mut versions = iter
		// TODO: Fail.
		.map(Result::unwrap)
		// Retrieve history as database records.
		.filter_map(|(k, v)| {
			let history_key = String::from_utf8(k.to_vec()).unwrap();
			if history_key.contains("VERSION")
				|| !history_key.starts_with(&format!("{}:", &slug))
			{
				// Skip VERSION records, ... or prefixes that match this slug?
				None
			} else {
				Some((HistoryKey(history_key), HistoryRecord::from_bytes(v)))
			}
		})
		// Map storage records to display records.
		.map(HistoryRevisionView::from)
		.collect::<Vec<_>>();

	// TODO: HACK.
	versions.reverse();

	Html(
		HistoryView {
			title: page.title(),
			slug: slug.as_str(),
			revisions: versions,
		}
		.render()
		.unwrap(),
	)
	.into_response()
}
