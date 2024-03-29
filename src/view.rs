use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};

use crate::{
	auth,
	auth::{
		namespace::Namespace,
		user::{User, UserView},
	},
	exists, not_found, ok,
	page::Page,
	Context,
};

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate<'a> {
	pub(crate) title: &'a str,
	pub(crate) body: &'a str,
	pub(crate) slug: &'a str,
	pub user: Option<UserView>,
}

pub async fn get(
	Path((ns, slug)): Path<(String, String)>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let user = ok!(User::authenticated(db, cookies).await);

	let ns = exists!(ok!(Namespace::get(db, &ns).await));
	if !ns.user_has_access(&user, auth::READ) {
		return not_found().await.into_response();
	}

	// TODO: Sanitize.
	if let Some(page) = Page::get(db, &ns.name, &slug).await {
		if !page.user_has_access(&user, &ns.name, auth::READ) {
			return not_found().await.into_response();
		}

		Html(
			ViewTemplate {
				title: page.title(),
				body: page.content(),
				slug: page.slug(),
				user: user.map(UserView::new),
			}
			.render()
			.unwrap(),
		)
		.into_response()
	} else {
		not_found().await.into_response()
	}
}
