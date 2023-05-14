use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{
	document::Document,
	history::db::{HistoryRecord, HistoryVersionRecord},
	not_found, Context, HIST_CF, PAGE_CF,
};

#[derive(Template)]
#[template(path = "edit.html")]
struct EditTemplate {
	title: String,
	content: String,
}

#[derive(Debug, Deserialize)]
pub struct EditPayload {
	content: String,
}

#[axum_macros::debug_handler]
pub async fn get(
	Path(slug): Path<String>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let doc = db
		// TODO: Sanitize.
		.get_cf(&db.cf_handle(PAGE_CF).unwrap(), &slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(doc) = doc {
		Html(
			EditTemplate {
				title: doc.title().clone(),
				content: doc.content().map(String::clone).unwrap_or_default(),
			}
			.render()
			.unwrap(),
		)
		.into_response()
	} else {
		not_found().await.into_response()
	}
}

#[axum_macros::debug_handler]
pub async fn post(
	Path(slug): Path<String>,
	State(ctx): State<Arc<Context>>,
	// TODO: Custom rejection.
	Form(params): Form<EditPayload>,
) -> impl IntoResponse {
	let Context { db, search } = ctx.as_ref();

	let hist_cf = db.cf_handle(HIST_CF).unwrap();
	let page_cf = db.cf_handle(PAGE_CF).unwrap();

	let doc = db
		// TODO: Sanitize.
		.get_cf(&page_cf, &slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(mut doc) = doc {
		// TODO: Move this.
		if let Some(current) = doc.content() {
			let tx = db.transaction();
			let version_key = HistoryVersionRecord::key(&slug);
			let version = tx
				// TODO: Brittle.
				.get_cf(&hist_cf, &version_key)
				.unwrap()
				.map(HistoryVersionRecord::from_bytes)
				.unwrap_or(HistoryVersionRecord::default());
			tx.put_cf(&hist_cf, version_key, version.next().as_bytes())
				.unwrap();
			tx.put_cf(
				&hist_cf,
				HistoryRecord::key(&slug, version),
				HistoryRecord::new(&slug, current, params.content.as_str())
					.as_bytes(),
			)
			.unwrap();
			tx.commit().unwrap();
		}

		// TODO: Sanitize.
		doc.set_content(params.content);

		// TODO: Handle DB error.
		db.put_cf(page_cf, doc.slug(), doc.as_bytes()).unwrap();

		search.write().unwrap().update_index(&doc);

		Redirect::to(&format!("/read/{slug}"))
	} else {
		Redirect::to(&format!("/write/{slug}?error=YES"))
	}
}
