use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;

use crate::{
	auth,
	auth::{namespace::Namespace, user::User, COOKIE_NAME},
	document::Document,
	encoding::{AsBytes, FromBytes},
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

	let doc = db
		// TODO: Sanitize.
		.get_cf(&db.cf_handle(PAGE_CF).unwrap(), &key)
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
	Path((ns, slug)): Path<(String, String)>,
	State(ctx): State<Arc<Context>>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	// TODO: Custom rejection.
	Form(params): Form<EditPayload>,
) -> impl IntoResponse {
	let Context { db, search } = ctx.as_ref();

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

	if !ns.user_has_access(&user, auth::WRITE) {
		return Redirect::to(&format!("/{}/{slug}/edit?error=EPERM", &ns.name))
			.into_response();
	}

	let key = format!("{}/{slug}", ns.name);

	let doc = db
		// TODO: Sanitize.
		.get_cf(&db.cf_handle(PAGE_CF).unwrap(), &key)
		// TODO: Handle DB error.
		.unwrap()
		.map(Document::from_bytes);

	if let Some(mut doc) = doc {
		// TODO: Move this.
		if let Some(current) = doc.content() {
			let tx = db.transaction();
			let version_key = HistoryVersionRecord::key(&ns.name, &slug);
			let version = tx
				// TODO: Brittle.
				.get_cf(&db.cf_handle(HIST_CF).unwrap(), &version_key)
				.unwrap()
				.map(HistoryVersionRecord::from_bytes)
				.unwrap_or(HistoryVersionRecord::default());
			tx.put_cf(
				&db.cf_handle(HIST_CF).unwrap(),
				version_key,
				version.next().as_bytes(),
			)
			.unwrap();
			tx.put_cf(
				&db.cf_handle(HIST_CF).unwrap(),
				HistoryRecord::key(&ns.name, &slug, version),
				HistoryRecord::new(
					&user.map(|u| u.name).unwrap_or("anonymous".to_string()),
					current,
					params.content.as_str(),
				)
				.as_bytes(),
			)
			.unwrap();
			tx.commit().unwrap();
		}

		// TODO: Sanitize.
		doc.set_content(params.content);

		// TODO: Handle DB error.
		db.put_cf(db.cf_handle(PAGE_CF).unwrap(), key, doc.as_bytes())
			.unwrap();

		search.write().unwrap().update_index(&ns.name, &doc);

		Redirect::to(&format!("/{}/{slug}", &ns.name)).into_response()
	} else {
		not_found().await.into_response()
	}
}
