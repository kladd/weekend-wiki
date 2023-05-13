use std::sync::Arc;

use axum::{
	http::StatusCode,
	response::{Html, IntoResponse},
	routing, Router,
};

mod create;
mod document;
mod edit;
mod view;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
const BINCODE_CONFIG: bincode::config::Configuration =
	bincode::config::standard();

type DB = Arc<rocksdb::DB>;

#[tokio::main]
async fn main() {
	let db = Arc::new(rocksdb::DB::open_default(LOCAL_DB_PATH).unwrap());

	let app = Router::new()
		.route("/", routing::get(index))
		.route("/create", routing::get(create::get))
		.route("/create", routing::post(create::post))
		.route("/read/:slug", routing::get(view::get))
		.route("/write/:slug", routing::get(edit::get))
		.route("/write/:slug", routing::post(edit::post))
		.fallback(not_found)
		.with_state(db);
	let server = tokio::net::TcpListener::bind("127.0.0.1:8080")
		.await
		.unwrap();

	axum::serve(server, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
	Html(INDEX_HTML)
}

async fn not_found() -> impl IntoResponse {
	(StatusCode::NOT_FOUND, "404: Not found")
}
