use axum::{
	http::{header::SET_COOKIE, HeaderMap},
	response::{IntoResponse, Redirect},
};
use axum_extra::{headers, TypedHeader};

use crate::auth::COOKIE_NAME;

#[axum_macros::debug_handler]
pub async fn get(
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
	let cookie = cookies.get(COOKIE_NAME);
	if let Some(username) = cookie {
		let cookie = format!("{}={}; Max-Age=0", COOKIE_NAME, username);

		let mut headers = HeaderMap::new();
		headers.insert(SET_COOKIE, cookie.parse().unwrap());
		(headers, Redirect::to("/login")).into_response()
	} else {
		Redirect::to("/login").into_response()
	}
}
