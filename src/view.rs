use std::sync::{Arc, LazyLock};

use askama::Template;
use axum::{
	extract::{Path, State},
	response::{Html, IntoResponse},
};
use axum_extra::{headers, TypedHeader};
use typst::{
	diag::{FileError, FileResult, Warned},
	foundations::{Bytes, Datetime},
	html::{HtmlDocument, HtmlElement, HtmlNode},
	syntax::{FileId, Source, VirtualPath},
	text::{Font, FontBook},
	utils::LazyHash,
	Feature, Features, Library, World,
};

use crate::{
	auth,
	auth::{
		namespace::Namespace,
		user::{User, UserView},
	},
	exists, not_found, ok,
	page::Page,
	Context,
};

#[derive(Template)]
#[template(path = "view.html", escape = "none")]
pub struct ViewTemplate<'a> {
	pub(crate) title: &'a str,
	pub(crate) body: &'a str,
	pub(crate) slug: &'a str,
	pub user: Option<UserView>,
}

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| {
	LazyHash::new(
		Library::builder()
			.with_features(vec![Feature::Html].into_iter().collect())
			.build(),
	)
});

static FONTS: LazyLock<LazyHash<FontBook>> =
	LazyLock::new(|| LazyHash::new(FontBook::new()));

struct SandboxWorld {
	file_id: FileId,
	source: Source,
}

impl SandboxWorld {
	pub fn new(text: &str) -> Self {
		let file_id = FileId::new(None, VirtualPath::new("main.typ"));
		let source = Source::new(file_id, text.to_string());
		Self { file_id, source }
	}
}

impl World for SandboxWorld {
	fn library(&self) -> &LazyHash<Library> {
		&LIBRARY
	}

	fn book(&self) -> &LazyHash<FontBook> {
		&FONTS
	}

	fn main(&self) -> FileId {
		self.file_id
	}

	fn source(&self, id: FileId) -> FileResult<Source> {
		if id == self.file_id {
			Ok(self.source.clone())
		} else {
			Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
		}
	}

	fn file(&self, id: FileId) -> FileResult<Bytes> {
		if id == self.file_id {
			Ok(self.source.text().as_bytes().into())
		} else {
			Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
		}
	}

	fn font(&self, _index: usize) -> Option<Font> {
		None
	}

	fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
		None
	}
}

pub async fn get(
	Path((ns, slug)): Path<(String, String)>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
	let Context { db, .. } = ctx.as_ref();

	let user = ok!(User::authenticated(db, cookies).await);

	let ns = exists!(ok!(Namespace::get(db, &ns).await));
	if !ns.user_has_access(&user, auth::READ) {
		return not_found().await.into_response();
	}

	// TODO: Sanitize.
	if let Some(page) = Page::get(db, &ns.name, &slug).await {
		if !page.user_has_access(&user, &ns.name, auth::READ) {
			return not_found().await.into_response();
		}

		let world = SandboxWorld::new(page.content());
		let Warned { output, warnings } =
			typst::compile::<HtmlDocument>(&world);
		// TODO: unwrap
		let result = output.and_then(|doc| typst_html::html(&doc)).unwrap();
		dbg!(warnings);

		Html(
			ViewTemplate {
				title: page.title(),
				body: &result,
				slug: page.slug(),
				user: user.map(UserView::new),
			}
			.render()
			.unwrap(),
		)
		.into_response()
	} else {
		not_found().await.into_response()
	}
}
