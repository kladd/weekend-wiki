use std::sync::Arc;

use axum::{
	extract::State,
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{document::Document, Context, CREATE_HTML, PAGE_CF};

#[derive(Deserialize)]
pub struct CreatePayload {
	title: String,
	namespace: String,
}

pub async fn get() -> impl IntoResponse {
	Html(CREATE_HTML)
}

#[axum_macros::debug_handler]
pub async fn post(
	State(state): State<Arc<Context>>,
	Form(params): Form<CreatePayload>,
) -> impl IntoResponse {
	// TODO: Sanitize.
	let doc = Document::new(params.title, None);

	let ns = params.namespace;
	let key = format!("{ns}/{}", doc.slug());

	// TODO: Check for duplicates first?
	state
		.db
		.put_cf(
			// TODO: Handles in context.
			state.db.cf_handle(PAGE_CF).unwrap(),
			key,
			doc.as_bytes(),
		)
		.unwrap();

	// TODO: Update search index.
	state.search.write().unwrap().update_index(&ns, &doc);

	Redirect::to(&format!("/{ns}/{}", doc.slug()))
}
