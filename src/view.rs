use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse, Redirect},
};
use axum_extra::{headers, TypedHeader};

use crate::{
	auth,
	auth::{namespace::Namespace, user::User, COOKIE_NAME},
	document::Document,
	encoding::FromBytes,
	not_found, Context, PAGE_CF,
};

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate<'a> {
	pub(crate) title: &'a str,
	pub(crate) body: &'a str,
	pub(crate) slug: &'a str,
}

pub async fn get(
	Path((ns, slug)): Path<(String, String)>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let ns = if let Some(ns) = Namespace::get(&db, &ns).await {
		ns
	} else {
		return not_found().await.into_response();
	};

	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(&db, username).await
	} else {
		None
	};

	if !ns.user_has_access(&user, auth::READ) {
		return not_found().await.into_response();
	}

	let key = format!("{}/{slug}", ns.name);

	let content = db
		// TODO: Sanitize.
		.get_cf(db.cf_handle(PAGE_CF).unwrap(), key)
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
