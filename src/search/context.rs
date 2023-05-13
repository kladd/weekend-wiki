use rocksdb::IteratorMode;
use tantivy::{
	collector::TopDocs,
	doc,
	query::QueryParser,
	schema::{Field, Schema, STORED, TEXT},
	Index, IndexWriter,
};

use crate::document::Document;

pub struct SearchContext {
	index: Index,
	index_writer: IndexWriter,
	query_parser: QueryParser,
	f_title: Field,
	f_slug: Field,
	f_content: Field,
}

pub struct QueryResult {
	pub(super) slug: String,
	pub(super) title: String,
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
			f_content: content,
		}
	}

	pub fn query(&self, query: &str) -> Vec<QueryResult> {
		// TODO: I think reader should be long lived?
		let searcher = self.index.reader().unwrap().searcher();
		// TODO: Sanitize.
		let q = self.query_parser.parse_query(query).unwrap();

		let search_results =
			searcher.search(&q, &TopDocs::with_limit(16)).unwrap();

		let mut results = vec![];
		for (_score, doc_address) in search_results {
			let doc = searcher.doc(doc_address).unwrap();
			results.push(QueryResult {
				slug: doc
					.get_first(self.f_slug)
					.unwrap()
					.as_text()
					.unwrap()
					.to_string(),
				title: doc
					.get_first(self.f_title)
					.unwrap()
					.as_text()
					.unwrap()
					.to_string(),
			})
		}

		results
	}

	pub fn update_index(&mut self, doc: &Document) {
		self.index_writer
			.add_document(doc!(
				self.f_slug => doc.slug().clone(),
				self.f_title => doc.title().clone(),
				self.f_content => doc.content().unwrap_or(&String::new()).as_str()
			))
			// TODO: Handle error.
			.unwrap();
		// TODO: Handle error.
		self.index_writer.commit().unwrap();
	}
}
