use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{document::Document, not_found, Context};

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
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let doc = db
		// TODO: Sanitize.
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
	State(ctx): State<Arc<Context>>,
	// TODO: Custom rejection.
	Form(params): Form<EditPayload>,
) -> impl IntoResponse {
	let Context { db, search } = ctx.as_ref();

	let doc = db
		// TODO: Sanitize.
		.get(&slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(mut doc) = doc {
		// TODO: Sanitize.
		doc.set_content(params.content);

		// TODO: Handle DB error.
		db.put(doc.slug(), doc.as_bytes()).unwrap();

		search.write().unwrap().update_index(&doc);

		Redirect::to(&format!("/read/{slug}"))
	} else {
		Redirect::to(&format!("/write/{slug}?error=YES"))
	}
}
