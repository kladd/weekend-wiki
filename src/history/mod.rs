use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};

use crate::{
	auth::user::{User, UserView},
	encoding::DbDecode,
	exists,
	history::{
		db::{HistoryKey, HistoryRecord},
		view::{HistoryRevisionView, HistoryView},
	},
	ok,
	page::Page,
	Context, HIST_CF,
};

pub mod db;
mod delta;
pub mod view;

#[axum_macros::debug_handler]
pub async fn get(
	// TODO: New type.
	Path((ns, slug)): Path<(String, String)>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let user = ok!(User::authenticated(db, cookies).await);
	let page = exists!(Page::get(db, &ns, &slug).await);
	let key = format!("{ns}/{slug}");

	let iter = db.prefix_iterator_cf(db.cf_handle(HIST_CF).unwrap(), &key);
	let mut versions = iter
		// TODO: Fail.
		.map(Result::unwrap)
		// Retrieve history as database records.
		.filter_map(|(k, v)| {
			let history_key = String::from_utf8(k.to_vec()).unwrap();
			if history_key.contains("VERSION")
				|| !history_key.starts_with(&format!("{}/", &key))
			{
				// Skip VERSION records, ... or prefixes that match this slug?
				None
			} else {
				Some((HistoryKey(history_key), HistoryRecord::dec(v)))
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
			user: user.map(UserView::new),
		}
		.render()
		.unwrap(),
	)
	.into_response()
}
