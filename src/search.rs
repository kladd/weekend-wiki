use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Query, State},
	response::{Html, IntoResponse},
};
use rocksdb::IteratorMode;
use serde::Deserialize;
use tantivy::{
	collector::TopDocs,
	doc,
	query::QueryParser,
	schema::{Field, Schema, STORED, TEXT},
	Index, IndexWriter,
};

use crate::{document::Document, Context};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
	#[serde(rename = "q")]
	query: String,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchResults {
	query: String,
	results: Vec<SearchResult>,
}

pub struct SearchResult {
	title: String,
	slug: String,
}

pub struct SearchContext {
	index: Index,
	index_writer: IndexWriter,
	query_parser: QueryParser,
	f_title: Field,
	f_slug: Field,
}

#[axum_macros::debug_handler]
pub async fn get(
	Query(params): Query<SearchParams>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { search, .. } = ctx.as_ref();

	// TODO: I think reader should be long lived?
	let searcher = search.index.reader().unwrap().searcher();
	// TODO: Sanitize.
	let query = search.query_parser.parse_query(&params.query).unwrap();

	let search_results =
		searcher.search(&query, &TopDocs::with_limit(16)).unwrap();
	let mut results = vec![];
	for (_score, doc_address) in search_results {
		let doc = searcher.doc(doc_address).unwrap();
		results.push(SearchResult {
			slug: doc
				.get_first(search.f_slug)
				.unwrap()
				.as_text()
				.unwrap()
				.to_string(),
			title: doc
				.get_first(search.f_title)
				.unwrap()
				.as_text()
				.unwrap()
				.to_string(),
		})
	}

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

impl SearchContext {
	const INDEX_SIZE_BYTES: usize = 0x300_000; // 3MB is the minimum.

	pub fn new(db: &rocksdb::DB) -> Self {
		let mut schema_builder = Schema::builder();
		let slug = schema_builder.add_text_field("slug", TEXT | STORED);
		let title = schema_builder.add_text_field("title", TEXT | STORED);
		let content = schema_builder.add_text_field("content", TEXT);
		let schema = schema_builder.build();
		let index = Index::create_in_ram(schema);
		let mut index_writer = index.writer(Self::INDEX_SIZE_BYTES).unwrap();

		// TODO: Obviously this won't scale forever, but I'm curious of how long
		//       it will.
		for (_, doc) in db.iterator(IteratorMode::Start).map(Result::unwrap) {
			let doc = Document::from_bytes(doc);
			index_writer
				.add_document(doc!(
					slug => doc.slug().as_str(),
					title => doc.title().as_str(),
					// TODO: These getters are terrible.
					content => doc.content().unwrap_or(&String::new()).as_str()
				))
				.unwrap();
		}
		index_writer.commit().unwrap();

		let query_parser = QueryParser::for_index(&index, vec![title, content]);

		Self {
			index,
			index_writer,
			query_parser,
			f_slug: slug,
			f_title: title,
		}
	}
}
