use std::sync::Arc;

use axum::{
	extract::State,
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{document::Document, search::SearchContext, Context, CREATE_HTML};

#[derive(Deserialize)]
pub struct CreatePayload {
	title: String,
	content: Option<String>,
}

pub async fn get() -> impl IntoResponse {
	Html(CREATE_HTML)
}

#[derive(Clone)]
pub struct CreateState {
	db: Arc<rocksdb::DB>,
	search: Arc<SearchContext>,
}

#[axum_macros::debug_handler]
pub async fn post(
	State(state): State<Arc<Context>>,
	Form(params): Form<CreatePayload>,
) -> impl IntoResponse {
	// TODO: Sanitize.
	let doc = Document::new(params.title, params.content);

	// TODO: Check for duplicates first?
	state.db.put(doc.slug().clone(), doc.as_bytes()).unwrap();

	// TODO: Update search index.

	Redirect::to(&format!("/read/{}", doc.slug()))
}
