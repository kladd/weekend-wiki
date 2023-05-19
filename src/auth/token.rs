use std::time::{SystemTime, UNIX_EPOCH};

use base58::{FromBase58, ToBase58};
use bincode::{Decode, Encode};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::encoding::{DbDecode, DbEncode};

const KEY: &[u8] = b"TODO: Secret key";

#[derive(Encode, Decode, Debug)]
pub struct Token {
	pub username: String,
	t: u64,
}

impl Token {
	pub fn new(username: &str) -> Self {
		Self {
			username: username.to_string(),
			t: SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap()
				.as_secs(),
		}
	}

	pub fn signed(&self) -> String {
		let mut mac = Hmac::<Sha256>::new_from_slice(KEY).unwrap();
		let encoded = self.enc();
		mac.update(&encoded);
		let hmac = mac.finalize().into_bytes();

		format!("{}.{}", encoded.to_base58(), hmac.to_base58())
	}

	pub fn verified(signed: &str) -> Option<Self> {
		let (token, signature) = signed.split_once('.')?;

		let mut mac = Hmac::<Sha256>::new_from_slice(KEY).unwrap();
		let token_enc = token.from_base58().ok()?;
		mac.update(&token_enc);
		mac.verify_slice(&signature.from_base58().ok()?).ok()?;

		Some(Token::dec(&token_enc))
	}
}
