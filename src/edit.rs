use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{document::Document, not_found, DB};

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

pub async fn get(
	Path(slug): Path<String>,
	State(db): State<DB>,
) -> impl IntoResponse {
	let doc = db
		.get(&slug)
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
	State(db): State<DB>,
	// TODO: Custom rejection.
	Form(params): Form<EditPayload>,
) -> impl IntoResponse {
	let doc = db
		.get(&slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(mut doc) = doc {
		doc.set_content(params.content);
		// TODO: Handle DB error.
		db.put(doc.slug(), doc.as_bytes()).unwrap();
		Redirect::to(&format!("/read/{slug}"))
	} else {
		Redirect::to(&format!("/write/{slug}?error=YES"))
	}
}
