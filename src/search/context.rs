use tantivy::{
	collector::TopDocs,
	doc,
	query::QueryParser,
	schema::{Facet, Field, Schema, STORED, TEXT},
	Index, IndexWriter, Term,
};

use crate::{
	encoding::DbDecode,
	page::{Page, PageKey},
};

pub struct SearchContext {
	index: Index,
	index_writer: IndexWriter,
	query_parser: QueryParser,
	f_path: Field,
	f_title: Field,
	f_slug: Field,
	f_content: Field,
}

pub struct QueryResult {
	pub(super) namespace: String,
	pub(super) slug: String,
	pub(super) title: String,
}

impl SearchContext {
	const INDEX_SIZE_BYTES: usize = 0x300_000; // 3MB is the minimum.

	pub async fn new(db: &rocksdb::TransactionDB) -> Self {
		let mut schema_builder = Schema::builder();
		let f_path = schema_builder.add_facet_field("path", STORED);
		let f_slug = schema_builder.add_text_field("slug", TEXT | STORED);
		let f_title = schema_builder.add_text_field("title", TEXT | STORED);
		let f_content = schema_builder.add_text_field("content", TEXT);
		let schema = schema_builder.build();
		let index = Index::create_in_ram(schema);
		let mut index_writer = index.writer(Self::INDEX_SIZE_BYTES).unwrap();

		// TODO: Obviously this won't scale forever, but I'm curious.
		// TODO: Do not unwrap.
		for (key, doc) in Page::list_all(db).await.map(Result::unwrap) {
			let path = PageKey::dec(key).as_facet();
			let doc = Page::dec(doc);
			index_writer
				.add_document(doc!(
					f_path => path,
					f_slug => doc.slug(),
					f_title => doc.title(),
					f_content => doc.content()
				))
				.unwrap();
		}
		index_writer.commit().unwrap();

		let query_parser =
			QueryParser::for_index(&index, vec![f_title, f_content]);

		Self {
			index,
			index_writer,
			query_parser,
			f_path,
			f_slug,
			f_title,
			f_content,
		}
	}

	// TODO: Any user can search anything and see it, fix that.
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
			let path = doc
				.get_first(self.f_path)
				.unwrap()
				.as_facet()
				.unwrap()
				.to_path_string();
			let (ns, slug) = &path[1..] // remove leading slash
				.split_once('/')
				.unwrap();
			results.push(QueryResult {
				namespace: ns.to_string(),
				slug: slug.to_string(),
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

	pub fn update_index(&mut self, ns: &str, doc: &Page) {
		// TODO: This is will remove all docs with this slug, not just the one
		//       in this namespace.
		let path = Facet::from_text(&format!("/{ns}/{}", doc.slug())).unwrap();
		self.index_writer
			.delete_term(Term::from_facet(self.f_path, &path));
		self.index_writer
			.add_document(doc!(
				self.f_path => path,
				self.f_slug => doc.slug().clone(),
				self.f_title => doc.title().clone(),
				self.f_content => doc.content()
			))
			// TODO: Handle error.
			.unwrap();

		// TODO: Handle error.
		self.index_writer.commit().unwrap();
	}
}
