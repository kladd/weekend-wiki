use std::sync::{Arc, RwLock};

use axum::{
	http::StatusCode,
	response::{Html, IntoResponse},
	routing, Router,
};
use rocksdb::{TransactionDB, TransactionDBOptions};

mod create;
mod document;
mod edit;
mod history;
mod search;
mod view;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
const BINCODE_CONFIG: bincode::config::Configuration =
	bincode::config::standard();
const PAGE_CF: &str = "page";
const HIST_CF: &str = "hist";

pub struct Context {
	// Database.
	db: TransactionDB,

	// Searching.
	search: RwLock<search::SearchContext>,
}

#[tokio::main]
async fn main() {
	// Database.
	// let db = rocksdb::DB::open_default(LOCAL_DB_PATH).unwrap();
	let mut db_opts = rocksdb::Options::default();
	db_opts.create_if_missing(true);
	db_opts.create_missing_column_families(true);
	let db = rocksdb::TransactionDB::open_cf(
		&db_opts,
		&TransactionDBOptions::default(),
		LOCAL_DB_PATH,
		vec![PAGE_CF, HIST_CF],
	)
	.unwrap();

	// Search
	let search_context = RwLock::new(search::SearchContext::new(&db));

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
		.route("/read/:slug/history", routing::get(history::get))
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