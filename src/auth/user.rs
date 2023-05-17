use std::collections::HashSet;

use bincode::{Decode, Encode};
use rocksdb::{IteratorMode, TransactionDB};

use crate::{
	encoding::{DbDecode, DbEncode},
	USER_CF,
};

#[derive(Debug)]
pub struct UserKey(String);

#[derive(Encode, Decode, Debug)]
pub struct User {
	// TODO: not pub
	pub name: String,
	pub password_hash: String,
	pub namespaces: HashSet<String>,
}

impl User {
	pub const META: &'static str = "meta";

	/// Creates an instance of a user which has access to the meta namespace and
	/// the user namespace.
	pub fn new(username: &str, password: &str) -> Self {
		let mut namespaces = HashSet::new();
		namespaces.insert("meta".to_string());
		namespaces.insert(username.to_string());

		Self {
			name: username.to_string(),
			password_hash: super_strong_password_hashing_algorithm(password),
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

// TODO: Obvious.
pub fn super_strong_password_hashing_algorithm(password: &str) -> String {
	password.to_string()
}

impl DbDecode for UserKey {
	fn dec<B: AsRef<[u8]>>(bytes: B) -> Self {
		Self(String::from_utf8(bytes.as_ref().to_vec()).unwrap())
	}
}
