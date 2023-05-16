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
	encoding::{DbDecode, DbEncode},
	history::db::{HistoryRecord, HistoryVersionRecord},
	not_found,
	page::Page,
	Context, HIST_CF,
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
	let ns = if let Some(ns) = Namespace::get(db, &ns).await {
		ns
	} else {
		return not_found().await.into_response();
	};

	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(db, username).await
	} else {
		None
	};

	if !ns.user_has_access(&user, auth::READ) {
		return not_found().await.into_response();
	}

	if let Some(page) = Page::get(db, &ns.name, &slug).await {
		Html(
			EditTemplate {
				title: page.title().to_string(),
				content: page.content().to_string(),
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

	let ns = if let Some(ns) = Namespace::get(db, &ns).await {
		ns
	} else {
		return not_found().await.into_response();
	};

	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(db, username).await
	} else {
		None
	};

	if !ns.user_has_access(&user, auth::WRITE) {
		return Redirect::to(&format!("/{}/{slug}/edit?error=EPERM", &ns.name))
			.into_response();
	}

	if let Some(mut page) = Page::get(db, &ns.name, &slug).await {
		// TODO: Move this.
		let tx = db.transaction();
		let version_key = HistoryVersionRecord::key(&ns.name, &slug);
		let version = tx
			// TODO: Brittle.
			.get_cf(&db.cf_handle(HIST_CF).unwrap(), &version_key)
			.unwrap()
			.map(HistoryVersionRecord::dec)
			.unwrap_or(HistoryVersionRecord::default());
		tx.put_cf(
			&db.cf_handle(HIST_CF).unwrap(),
			version_key,
			version.next().enc(),
		)
		.unwrap();
		tx.put_cf(
			&db.cf_handle(HIST_CF).unwrap(),
			HistoryRecord::key(&ns.name, &slug, version),
			HistoryRecord::new(
				&user.map(|u| u.name).unwrap_or("anonymous".to_string()),
				page.content(),
				params.content.as_str(),
			)
			.enc(),
		)
		.unwrap();
		tx.commit().unwrap();

		// TODO: Sanitize.
		page.set_content(params.content.as_str());

		// TODO: Handle DB error.
		Page::put(db, &ns.name, &page).await;

		search.write().unwrap().update_index(&ns.name, &page);

		Redirect::to(&format!("/{}/{slug}", &ns.name)).into_response()
	} else {
		not_found().await.into_response()
	}
}
