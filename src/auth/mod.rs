use rocksdb::TransactionDB;

use crate::{
	auth::{namespace::Namespace, user::User},
	errors::WkError,
};

pub mod login;
pub mod logout;
pub mod namespace;
pub mod user;

pub const COOKIE_NAME: &str = "SESSION";

// Requests / Modes.
pub const MANAGE: u16 = 0o1;
pub const READ: u16 = 0o4;
pub const WRITE: u16 = 0o2;
pub const MASK: u16 = 0o7;

// Kinds.
const OWNER: u16 = 6;
const NAMESPACE: u16 = 3;
const OTHERS: u16 = 0;

pub fn has_access(mode: u16, kind: u16, request: u16) -> bool {
	((mode >> kind) & MASK) & request != 0
}

// TODO: Result.
pub async fn add_user_to_namespace(
	db: &TransactionDB,
	user: &mut User,
	namespace: &mut Namespace,
) -> Result<(), WkError> {
	namespace.members.insert(user.name.clone());
	user.namespaces.insert(namespace.name.clone());
	// TODO: Transaction.
	User::put(db, user).await;
	Namespace::put(db, namespace).await
}
