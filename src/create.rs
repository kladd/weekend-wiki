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
	auth::{
		add_user_to_namespace, namespace::Namespace, user::User, COOKIE_NAME,
	},
	document::Document,
	encoding::AsBytes,
	Context, CREATE_HTML, PAGE_CF,
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
	// TODO: Rejection and all.
	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(&state.db, username).await
	} else {
		None
	};

	let ns = {
		if let Some(ns) = Namespace::get(&state.db, &params.namespace).await {
			if !ns.user_has_access(&user, auth::WRITE) {
				return Redirect::to("/create?error=EPERM").into_response();
			}
			ns
		} else if let Some(user) = user {
			// TODO: Any user can create a namespace, but I could see an admin
			//       not wanting that.
			// TODO: Also one must be a user to create a namespace, this too may
			//       not be desirable.
			let mut user = user;
			let mut new_ns =
				Namespace::new(&params.namespace, &user.name, 0o755);
			add_user_to_namespace(&state.db, &mut user, &mut new_ns).await;
			new_ns
		} else {
			return Redirect::to("/login").into_response();
		}
	};

	// TODO: Custom perms.
	// TODO: Sanitize.
	let doc = Document::new(params.title, 0o644, None);

	let key = format!("{}/{}", ns.name, doc.slug());

	// TODO: Check for duplicates first?
	state
		.db
		.put_cf(
			// TODO: Handles in context.
			state.db.cf_handle(PAGE_CF).unwrap(),
			key,
			doc.as_bytes(),
		)
		.unwrap();

	// TODO: Update search index.
	state.search.write().unwrap().update_index(&ns.name, &doc);

	Redirect::to(&format!("/{}/{}", ns.name, doc.slug())).into_response()
}
