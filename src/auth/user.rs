use std::collections::HashSet;

use axum_extra::headers;
use bincode::{Decode, Encode};
use password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use pbkdf2::Pbkdf2;
use rocksdb::{IteratorMode, TransactionDB};

use crate::{
	auth::{token::Token, COOKIE_NAME},
	encoding::{DbDecode, DbEncode},
	errors::WkError,
	USER_CF,
};

#[derive(Encode, Decode, Debug)]
pub struct UserKey(String);

#[derive(Encode, Decode, Debug)]
pub struct User {
	// TODO: not pub
	pub name: String,
	pub password_hash: String,
	pub namespaces: HashSet<String>,
}

pub struct UserView {
	pub name: String,
}

impl User {
	pub const META: &'static str = "meta";

	/// Creates an instance of a user which has access to the meta namespace and
	/// the user namespace.
	pub fn new(username: &str, password: &str) -> Self {
		let mut namespaces = HashSet::new();
		namespaces.insert("meta".to_string());
		namespaces.insert(username.to_string());

		let salt = SaltString::generate(&mut OsRng);
		let hash = PasswordHash::generate(Pbkdf2, password, &salt).unwrap();

		Self {
			name: username.to_string(),
			password_hash: hash.to_string(),
			namespaces,
		}
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub async fn get(db: &TransactionDB, name: &str) -> Option<User> {
		let cf = db.cf_handle(USER_CF).unwrap();
		let bytes = db.get_cf(&cf, name).unwrap()?;
		Some(User::dec(bytes))
	}

	pub async fn authenticated(
		db: &TransactionDB,
		cookie: headers::Cookie,
	) -> Result<Option<Self>, WkError> {
		let token = cookie.get(COOKIE_NAME).and_then(Token::verified);
		let user = match token {
			Some(Token { username, .. }) => User::get(db, &username).await,
			None => None,
		};
		Ok(user)
	}

	pub async fn put(db: &TransactionDB, user: &Self) {
		let cf = db.cf_handle(USER_CF).unwrap();
		db.put_cf(&cf, &user.name, user.enc()).unwrap()
	}

	pub async fn list(db: &TransactionDB) -> Vec<User> {
		let cf = db.cf_handle(USER_CF).unwrap();
		let iter = db.full_iterator_cf(&cf, IteratorMode::Start);

		// TODO: Handle the errors.
		iter.flatten().map(|(_, v)| User::dec(v)).collect()
	}
}

impl UserView {
	pub fn new(user: User) -> Self {
		Self { name: user.name }
	}
}
