use std::sync::Arc;

use askama::Template;
use axum::{
	extract::State,
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};

use crate::{
	auth::user::{User, UserView},
	ok, Context,
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
	let user = ok!(User::authenticated(db, cookies).await);

	Html(
		IndexView {
			user: user.map(UserView::new),
		}
		.render()
		.unwrap(),
	)
	.into_response()
}
