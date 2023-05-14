use std::{
	fs,
	path::Path,
	sync::{Arc, RwLock},
};

use axum::{
	extract::State,
	http::StatusCode,
	response::{Html, IntoResponse},
	routing, Router,
};
use rocksdb::{IteratorMode, TransactionDB, TransactionDBOptions};
use slug::slugify;
use tower_http::services::ServeDir;

use crate::{
	document::{Document, DocumentKey},
	history::db::{HistoryKey, HistoryVersionRecord},
};

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

	// Populate meta namespace.
	// TODO: This really doesn't need to happen every time the application
	// starts.
	seed_base(&db);

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
		.route("/:ns/:slug", routing::get(view::get))
		.route("/:ns/:slug/history", routing::get(history::get))
		.route("/:ns/:slug/edit", routing::get(edit::get))
		.route("/:ns/:slug/edit", routing::post(edit::post))
		.route("/dump", routing::get(dump))
		.nest_service("/dist", ServeDir::new("dist"))
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

#[axum_macros::debug_handler]
async fn dump(State(ctx): State<Arc<Context>>) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let option = db.cf_handle(PAGE_CF).unwrap();
	let pages = db.full_iterator_cf(&option, IteratorMode::Start);
	for page in pages {
		let (k, v) = page.unwrap();
		println!(
			"PAGE {:?} => {}",
			DocumentKey::from_bytes(k),
			Document::from_bytes(v).title()
		);
	}

	let history = db
		.full_iterator_cf(&db.cf_handle(HIST_CF).unwrap(), IteratorMode::Start);
	for hist in history {
		let (k, v) = hist.unwrap();
		let key = HistoryKey::from_bytes(k);
		if key.revision().contains("VERSION") {
			println!(
				"HIST {key:?} => {:?}",
				HistoryVersionRecord::from_bytes(v)
			);
		} else {
			println!("HIST {key:?} => [DIFF]");
		}
	}

	(StatusCode::OK, "OK")
}

/// Create pages from the `base` dir.
fn seed_base(db: &TransactionDB) {
	let page_db = db.cf_handle(PAGE_CF).unwrap();
	for dir in fs::read_dir("base").unwrap() {
		let dir_path = dir.unwrap().path();
		let namespace = dir_path.file_name().unwrap();

		for file in fs::read_dir(&dir_path)
			.expect("I hope you don't have files in the base directory")
		{
			let firent = file.unwrap();
			let fname = firent.file_name();
			let title =
				Path::file_stem(fname.as_ref()).unwrap().to_str().unwrap();
			let content = fs::read_to_string(firent.path());

			db.put_cf(
				&page_db,
				format!("{}/{}", namespace.to_str().unwrap(), slugify(title))
					.as_bytes(),
				Document::new(title.to_string(), Some(content.unwrap()))
					.as_bytes(),
			)
			.unwrap();
		}
	}
}
