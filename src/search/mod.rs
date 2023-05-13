mod context;

use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Query, State},
	response::{Html, IntoResponse},
};
pub use context::SearchContext;
use serde::Deserialize;

use crate::Context;

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
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { search, .. } = ctx.as_ref();

	let results = search.read().unwrap().query(&params.query);

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
