use std::sync::Arc;

use axum::{
	extract::State,
	http::{header::SET_COOKIE, HeaderMap},
	response::{Html, IntoResponse, Redirect},
	Form,
};
use password_hash::{PasswordHash, PasswordVerifier};
use pbkdf2::Pbkdf2;
use serde::Deserialize;

use crate::{
	auth::{token::Token, user::User, COOKIE_NAME},
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
			Pbkdf2
				.verify_password(
					params.password.as_bytes(),
					&PasswordHash::new(&found.password_hash).unwrap(),
				)
				.is_ok()
		});

	if let Some(user) = user {
		let token = Token::new(user.name());

		let cookie = format!(
			"{}={}; SameSite=Strict; Path=/; HttpOnly",
			COOKIE_NAME,
			token.signed()
		);

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
