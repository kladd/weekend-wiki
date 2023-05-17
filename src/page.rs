use bincode::{Decode, Encode};
use rocksdb::{DBIteratorWithThreadMode, IteratorMode, TransactionDB};
use slug::slugify;
use tantivy::schema::Facet;

use crate::{
	auth,
	auth::{has_access, user::User},
	encoding::{DbDecode, DbEncode},
	PAGE_CF,
};

#[repr(transparent)]
#[derive(Encode, Decode, Debug)]
pub struct PageKey(String);

#[derive(Encode, Decode)]
pub struct Page {
	title: String,
	slug: String,
	mode: u16,
	content: String,
	owner: Option<String>,
}

impl Page {
	pub const DEFAULT_MODE: u16 = 0o666;

	// TODO: Better signature.
	pub fn new(
		title: &str,
		mode: u16,
		owner: Option<&str>,
		content: Option<String>,
	) -> Self {
		Self {
			mode,
			title: title.to_string(),
			owner: owner.map(str::to_string),
			// TODO: I don't really like that this doesn't retain
			//       capitalization. Write a slugger like Wikipedia, e.g.
			//       https://en.wikipedia.org/wiki/Clean_URL
			slug: slugify(title),
			content: content.unwrap_or_default(),
		}
	}

	pub fn slug(&self) -> &str {
		&self.slug
	}

	pub fn title(&self) -> &str {
		&self.title
	}

	pub fn content(&self) -> &str {
		&self.content
	}

	pub fn set_content(&mut self, content: &str) {
		self.content = content.to_string()
	}

	pub async fn get(db: &TransactionDB, ns: &str, slug: &str) -> Option<Self> {
		let key = PageKey::new(ns, slug);
		db.get_cf(&db.cf_handle(PAGE_CF).unwrap(), key.enc())
			.unwrap()
			.map(Page::dec)
	}

	pub async fn put(db: &TransactionDB, ns: &str, page: &Self) {
		let key = PageKey::new(ns, &page.slug);
		db.put_cf(&db.cf_handle(PAGE_CF).unwrap(), key.enc(), page.enc())
			.unwrap()
	}

	pub async fn list<'a>(
		db: &'a TransactionDB,
		ns: &str,
	) -> impl Iterator + 'a {
		db.prefix_iterator_cf(
			&db.cf_handle(PAGE_CF).unwrap(),
			format!("{ns}/").enc(),
		)
	}

	pub async fn list_all(
		db: &TransactionDB,
	) -> DBIteratorWithThreadMode<TransactionDB> {
		db.full_iterator_cf(
			&db.cf_handle(PAGE_CF).unwrap(),
			IteratorMode::Start,
		)
	}

	pub fn user_has_access(
		&self,
		user: &Option<User>,
		namespace: &str,
		access: u16,
	) -> bool {
		// TODO: Duplicated logic with Namespace.
		let owner_group = if let Some(user) = user {
			if user.name == User::META {
				true
			} else {
				// TODO: Right now the owner always has full control. Is that
				//       what we want?
				let owner =
					// Users never have owner permissions if the page has no owner?
					self.owner.as_ref().filter(|owner| **owner == user.name).is_some();
				let group = user.namespaces.contains(namespace)
					&& has_access(self.mode, auth::NAMESPACE, access);
				owner || group
			}
		} else {
			false
		};
		owner_group || has_access(self.mode, auth::OTHERS, access)
	}
}

impl PageKey {
	pub fn new(ns: &str, slug: &str) -> Self {
		// Slug goes first because the most variable segment of the key.
		Self(format!("{ns}/{slug}"))
	}

	pub fn as_facet(&self) -> Facet {
		Facet::from_text(&format!("/{}", self.0)).unwrap()
	}
}
