mod context;

use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Query, State},
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};
pub use context::SearchContext;
use serde::Deserialize;

use crate::{
	auth,
	auth::{namespace::Namespace, user::User, COOKIE_NAME},
	Context,
};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
	#[serde(rename = "q")]
	query: String,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchResults {
	query: String,
	results: Vec<context::QueryResult>,
}

#[axum_macros::debug_handler]
pub async fn get(
	Query(params): Query<SearchParams>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { search, db, .. } = ctx.as_ref();

	let user = if let Some(username) = cookies.get(COOKIE_NAME) {
		User::get(db, username).await
	} else {
		None
	};

	let namespaces = Namespace::list_with_access(db, user, auth::READ).await;
	let ns_names = namespaces
		.iter()
		.map(|ns| ns.name.as_str())
		.collect::<Vec<_>>();

	let results = search.read().unwrap().query(&params.query, ns_names);

	Html(
		SearchResults {
			// TODO: Sanitize.
			query: params.query,
			results,
		}
		.render()
		.unwrap(),
	)
}
