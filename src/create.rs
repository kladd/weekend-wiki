use axum::{
	extract::State,
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{document::Document, CREATE_HTML, DB};

#[derive(Deserialize)]
pub struct CreatePayload {
	title: String,
	content: Option<String>,
}

pub async fn get() -> impl IntoResponse {
	Html(CREATE_HTML)
}

pub async fn post(
	State(db): State<DB>,
	Form(params): Form<CreatePayload>,
) -> impl IntoResponse {
	let doc = Document::new(params.title, params.content);

	// TODO: Check for duplicates first?
	db.put(doc.slug().clone(), doc.as_bytes()).unwrap();

	Redirect::to(&format!("/read/{}", doc.slug()))
}
