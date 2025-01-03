use std::{
	fs,
	path::Path,
	sync::{Arc, RwLock},
};

use axum::{
	extract::State, http::StatusCode, response::IntoResponse, routing, Router,
};
use rocksdb::{IteratorMode, TransactionDB, TransactionDBOptions};
use tower_http::services::ServeDir;
use tracing::info;

use crate::{
	auth::{
		add_user_to_namespace,
		namespace::{Namespace, NamespaceKey},
		user::{User, UserKey},
	},
	encoding::DbDecode,
	history::db::{HistoryKey, HistoryVersionRecord},
	page::{Page, PageKey},
};

mod auth;
mod control;
mod create;
mod edit;
mod encoding;
mod errors;
mod history;
mod index;
mod page;
mod search;
mod view;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
const BINCODE_CONFIG: bincode::config::Configuration =
	bincode::config::standard();

// COLUMN NAMES MUST BE 4 CHARACTERS LONG. NOT FOR ANY TECHNICAL REASON, BUT
// THIS ALIGNMENT MUST NOT BE BROKEN, OR ELSE.
const PAGE_CF: &str = "page";
const HIST_CF: &str = "hist";
const NSPC_CF: &str = "nspc";
const USER_CF: &str = "user";

pub struct Context {
	// Database.
	db: TransactionDB,

	// Searching.
	search: RwLock<search::SearchContext>,
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	info!("Starting with database: {LOCAL_DB_PATH}");
	// Database.
	let mut db_opts = rocksdb::Options::default();
	db_opts.create_if_missing(true);
	db_opts.create_missing_column_families(true);
	let db = rocksdb::TransactionDB::open_cf(
		&db_opts,
		&TransactionDBOptions::default(),
		LOCAL_DB_PATH,
		vec![PAGE_CF, HIST_CF, USER_CF, NSPC_CF],
	)
	.unwrap();

	// Populate meta namespace.
	// TODO: This really doesn't need to happen every time the application
	//       starts.
	info!("Seeding database");
	seed_base(&db).await;

	// Search
	info!("Building search index");
	let search_context = RwLock::new(search::SearchContext::new(&db).await);

	// Whole world.
	let context = Arc::new(Context {
		db,
		search: search_context,
	});

	// Web pages.
	let app = Router::new()
		.route("/", routing::get(index::get))
		.route("/search", routing::get(search::get))
		.route("/create", routing::get(create::get))
		.route("/create", routing::post(create::post))
		.route("/:ns/:slug", routing::get(view::get))
		.route("/:ns/:slug/history", routing::get(history::get))
		.route("/:ns/:slug/edit", routing::get(edit::get))
		.route("/:ns/:slug/edit", routing::post(edit::post))
		.route("/login", routing::get(auth::login::get))
		.route("/login", routing::post(auth::login::post))
		.route("/logout", routing::get(auth::logout::get))
		.route("/control", routing::get(control::get))
		.route("/control", routing::post(control::post))
		.route("/dump", routing::get(dump))
		.nest_service("/dist", ServeDir::new("dist"))
		.fallback(not_found)
		.with_state(context);

	let addr = "0.0.0.0:8080";
	let server = tokio::net::TcpListener::bind(addr).await.unwrap();

	info!("Listening on {addr}");
	axum::serve(server, app).await.unwrap();
}

async fn not_found() -> impl IntoResponse {
	(StatusCode::NOT_FOUND, "404: Not found")
}

#[axum_macros::debug_handler]
async fn dump(State(ctx): State<Arc<Context>>) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let pages = db
		.full_iterator_cf(&db.cf_handle(PAGE_CF).unwrap(), IteratorMode::Start);
	for page in pages {
		let (k, v) = page.unwrap();
		info!("PAGE {:?} => {}", PageKey::dec(k), Page::dec(v).title());
	}

	let history = db
		.full_iterator_cf(&db.cf_handle(HIST_CF).unwrap(), IteratorMode::Start);
	for hist in history {
		let (k, v) = hist.unwrap();
		let key = HistoryKey::from_bytes(k);
		if key.revision().contains("VERSION") {
			info!("HIST {key:?} => {:?}", HistoryVersionRecord::dec(v));
		} else {
			info!("HIST {key:?} => [DIFF]");
		}
	}

	let nss = db
		.full_iterator_cf(&db.cf_handle(NSPC_CF).unwrap(), IteratorMode::Start);
	for ns in nss {
		let (k, v) = ns.unwrap();
		info!("NSPC {:?} => {:?}", NamespaceKey::dec(k), Namespace::dec(v));
	}

	let users = db
		.full_iterator_cf(&db.cf_handle(USER_CF).unwrap(), IteratorMode::Start);
	for user in users {
		let (k, v) = user.unwrap();
		info!("USER {:?} => {:?}", UserKey::dec(k), User::dec(v));
	}

	(StatusCode::OK, "OK")
}

/// Create pages from the `base` dir.
async fn seed_base(db: &TransactionDB) {
	let mut meta_user = User::new(User::META, "default");
	let mut meta_ns = Namespace::new(User::META, User::META, 0o744);
	// Panics: We're initializing, so prefer to crash here.
	add_user_to_namespace(db, &mut meta_user, &mut meta_ns)
		.await
		.unwrap();

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

			Page::put(
				db,
				namespace.to_str().unwrap(),
				&Page::new(title, 0o644, Some("meta"), Some(content.unwrap())),
			)
			.await
		}
	}
}
