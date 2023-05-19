use std::sync::Arc;

use axum::{
	extract::State,
	response::{Html, IntoResponse, Redirect},
	Form,
};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;

use crate::{
	auth,
	auth::{add_user_to_namespace, namespace::Namespace, user::User},
	ok,
	page::Page,
	Context, CREATE_HTML,
};

#[derive(Deserialize)]
pub struct CreatePayload {
	title: String,
	namespace: String,
}

pub async fn get() -> impl IntoResponse {
	Html(CREATE_HTML)
}

#[axum_macros::debug_handler]
pub async fn post(
	State(state): State<Arc<Context>>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	Form(params): Form<CreatePayload>,
) -> impl IntoResponse {
	let mut user = ok!(User::authenticated(&state.db, cookies).await);

	let ns = {
		let ns_maybe = ok!(Namespace::get(&state.db, &params.namespace).await);
		if let Some(ns) = ns_maybe {
			if !ns.user_has_access(&user, auth::WRITE) {
				return Redirect::to("/create?error=EPERM").into_response();
			}
			ns
		} else if let Some(ref mut user) = user {
			// TODO: Any user can create a namespace, but I could see an admin
			//       not wanting that.
			// TODO: Also one must be a user to create a namespace, this too may
			//       not be desirable.
			let mut new_ns =
				Namespace::new(&params.namespace, &user.name, 0o755);
			ok!(add_user_to_namespace(&state.db, user, &mut new_ns).await);
			new_ns
		} else {
			return Redirect::to("/login").into_response();
		}
	};

	// TODO: Custom perms.
	// TODO: Sanitize.
	let page = Page::new(
		params.title.as_str(),
		Page::DEFAULT_MODE - ns.umask,
		user.as_ref().map(User::name),
		None,
	);

	// TODO: Check for duplicates first.
	Page::put(&state.db, &ns.name, &page).await;

	// TODO: Update search index.
	state.search.write().unwrap().update_index(&ns.name, &page);

	Redirect::to(&format!("/{}/{}", ns.name, page.slug())).into_response()
}
