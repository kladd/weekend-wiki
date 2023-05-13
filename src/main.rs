use std::sync::Arc;

use axum::{
	http::StatusCode,
	response::{Html, IntoResponse},
	routing, Router,
};


mod create;
mod document;
mod edit;
mod search;
mod view;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
const BINCODE_CONFIG: bincode::config::Configuration =
	bincode::config::standard();

type DB = Arc<rocksdb::DB>;

pub struct Context {
	// Database.
	db: rocksdb::DB,

	// Searching.
	search: search::SearchContext,
}

#[tokio::main]
async fn main() {
	// Database.
	let db = rocksdb::DB::open_default(LOCAL_DB_PATH).unwrap();

	// Search
	let search_context = search::SearchContext::new(&db);

	// Whole world.
	let context = Arc::new(Context {
		db,
		search: search_context,
	});

	// Web pages.
	let app = Router::new()
		.route("/", routing::get(get))
		.route("/search", routing::get(search::get))
		.route("/create", routing::get(create::get))
		.route("/create", routing::post(create::post))
		.route("/read/:slug", routing::get(view::get))
		.route("/write/:slug", routing::get(edit::get))
		.route("/write/:slug", routing::post(edit::post))
		.fallback(not_found)
		.with_state(context);
	let server = tokio::net::TcpListener::bind("127.0.0.1:8080")
		.await
		.unwrap();

	axum::serve(server, app).await.unwrap();
}

async fn get() -> Html<&'static str> {
	Html(INDEX_HTML)
}

async fn not_found() -> impl IntoResponse {
	(StatusCode::NOT_FOUND, "404: Not found")
}
