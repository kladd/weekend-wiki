use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{document::Document, not_found, Context, PAGE_CF};

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

	let page_cf = db.cf_handle(PAGE_CF).unwrap();

	let doc = db
		// TODO: Sanitize.
		.get_cf(&page_cf, &slug)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(mut doc) = doc {
		// TODO: Store this.
		if let Some(current) = doc.content() {
			for (line, ch) in
				diff::lines(current, &params.content).iter().enumerate()
			{
				match ch {
					diff::Result::Left(l) => println!("{line}-{l}"),
					diff::Result::Both(l, _) => (),
					diff::Result::Right(r) => println!("{line}+{r}"),
				}
			}
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
