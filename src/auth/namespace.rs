use std::collections::HashSet;

use bincode::{Decode, Encode};
use rocksdb::{IteratorMode, TransactionDB};

use crate::{
	auth,
	auth::{has_access, user::User},
	encoding::{DbDecode, DbEncode},
	errors::WkError,
	NSPC_CF,
};

#[derive(Encode, Decode, Debug)]
pub struct NamespaceKey(String);

// Part Unix group, part Unix directory.
#[derive(Encode, Decode, Debug)]
pub struct Namespace {
	// TODO: Not pub
	pub mode: u16,
	pub umask: u16,
	pub name: String,
	pub owner: String,
	pub members: HashSet<String>,
}

impl Namespace {
	pub const DEFAULT_MODE: u16 = 0o777;
	pub const DEFAULT_UMASK: u16 = 0o022;

	pub fn new(name: &str, owner: &str, mode: u16) -> Self {
		Self {
			name: name.to_string(),
			mode,
			owner: owner.to_string(),
			umask: Self::DEFAULT_UMASK,
			members: HashSet::new(),
		}
	}

	pub async fn get(
		db: &TransactionDB,
		name: &str,
	) -> Result<Option<Namespace>, WkError> {
		let cf = db.cf_handle(NSPC_CF).unwrap();
		Ok(db.get_cf(&cf, name)?.map(Namespace::dec))
	}

	pub async fn put(db: &TransactionDB, ns: &Self) -> Result<(), WkError> {
		let cf = db.cf_handle(NSPC_CF).unwrap();
		db.put_cf(&cf, &ns.name, ns.enc()).map_err(WkError::from)
	}

	pub async fn list(db: &TransactionDB) -> Vec<Namespace> {
		let cf = db.cf_handle(NSPC_CF).unwrap();
		let iter = db.full_iterator_cf(&cf, IteratorMode::Start);

		// TODO: Handle the errors.
		iter.flatten().map(|(_, v)| Namespace::dec(v)).collect()
	}

	pub fn user_has_access(&self, user: &Option<User>, access: u16) -> bool {
		let owner_group = if let Some(user) = user {
			if user.name == User::META {
				true
			} else {
				let owner = self.owner == user.name;
				let group = self.members.contains(&user.name)
					&& has_access(self.mode, auth::NAMESPACE, access);
				owner || group
			}
		} else {
			false
		};
		owner_group || has_access(self.mode, auth::OTHERS, access)
	}

	pub async fn list_with_access(
		db: &TransactionDB,
		user: Option<User>,
		access: u16,
	) -> Vec<Namespace> {
		Self::list(db)
			.await
			.into_iter()
			.filter(|ns| ns.user_has_access(&user, access))
			.collect()
	}
}
