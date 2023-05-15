use std::sync::Arc;

use axum::{
	extract::State,
	http::{header::SET_COOKIE, HeaderMap},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use serde::Deserialize;

use crate::{
	auth::{
		user::{super_strong_password_hashing_algorithm, User},
		COOKIE_NAME,
	},
	Context, LOGIN_HTML,
};

#[derive(Deserialize)]
pub struct LoginPayload {
	username: String,
	password: String,
}

#[axum_macros::debug_handler]
pub async fn post(
	State(state): State<Arc<Context>>,
	Form(params): Form<LoginPayload>,
) -> impl IntoResponse {
	let user = User::get(&state.db, &params.username)
		.await
		.filter(|found| {
			found.password_hash
				== super_strong_password_hashing_algorithm(&params.password)
		});

	if let Some(user) = user {
		// TODO: All very, very secure.
		let cookie =
			format!("{}={}; SameSite=Lax; Path=/", COOKIE_NAME, user.name);

		// Set cookie
		let mut headers = HeaderMap::new();
		headers.insert(SET_COOKIE, cookie.parse().unwrap());

		(headers, Redirect::to("/")).into_response()
	} else {
		Redirect::to("/login?error=ENOENT").into_response()
	}
}

pub async fn get() -> impl IntoResponse {
	Html(LOGIN_HTML)
}
