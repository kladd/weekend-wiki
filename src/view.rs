use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};

use crate::{
	auth,
	auth::{namespace::Namespace, user::User, COOKIE_NAME},
	not_found,
	page::Page,
	resource_or_return_error, Context,
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

	let ns = resource_or_return_error!(Namespace::get(db, &ns).await);

	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(db, username).await
	} else {
		None
	};

	if !ns.user_has_access(&user, auth::READ) {
		return not_found().await.into_response();
	}

	// TODO: Sanitize.
	let content = Page::get(db, &ns.name, &slug).await;

	if let Some(doc) = content {
		Html(
			ViewTemplate {
				title: doc.title(),
				body: doc.content(),
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
