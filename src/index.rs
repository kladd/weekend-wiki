use std::sync::Arc;

use askama::Template;
use axum::{
	extract::State,
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};

use crate::{
	auth::{user::User, UserView, COOKIE_NAME},
	Context,
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexView {
	user: Option<UserView>,
}

pub async fn get(
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();
	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(db, username).await
	} else {
		None
	};

	Html(
		IndexView {
			user: user.map(UserView::new),
		}
		.render()
		.unwrap(),
	)
	.into_response()
}
