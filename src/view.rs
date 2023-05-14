use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};

use crate::{document::Document, not_found, Context, PAGE_CF};

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate<'a> {
	pub(crate) title: &'a str,
	pub(crate) body: &'a str,
	pub(crate) slug: &'a str,
}

pub async fn get(
	Path(slug): Path<String>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let content = db
		// TODO: Sanitize.
		.get_cf(db.cf_handle(PAGE_CF).unwrap(), &slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(doc) = content {
		Html(
			ViewTemplate {
				title: doc.title(),
				body: doc.content().map(String::as_str).unwrap_or(""),
				slug: doc.slug(),
			}
			.render()
			.unwrap(),
		)
		.into_response()
	} else {
		not_found().await.into_response()
	}
}
