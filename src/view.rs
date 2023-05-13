use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};

use crate::{document::Document, not_found, DB};

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate {
	pub(crate) title: String,
	pub(crate) body: String,
	pub(crate) slug: String,
}

pub async fn get(
	Path(slug): Path<String>,
	State(db): State<DB>,
) -> impl IntoResponse {
	let content = db
		.get(&slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(doc) = content {
		Html(
			ViewTemplate {
				title: doc.title().clone(),
				body: doc.content().map(String::clone).unwrap_or_default(),
				slug: doc.slug().clone(),
			}
			.render()
			.unwrap(),
		)
		.into_response()
	} else {
		not_found().await.into_response()
	}
}
