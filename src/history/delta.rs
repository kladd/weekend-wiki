use std::fmt::{Display, Formatter};

use bincode::{Decode, Encode};

const DIFF_CONTEXT: usize = 3;

#[derive(Encode, Decode, Debug)]
pub struct Delta(String);

impl Delta {
	pub fn new<L, R>(slug: &str, prev: L, next: R) -> Self
	where
		L: AsRef<str>,
		R: AsRef<str>,
	{
		Self(
			String::from_utf8(unified_diff::diff(
				prev.as_ref().as_bytes(),
				slug,
				next.as_ref().as_bytes(),
				slug,
				DIFF_CONTEXT,
			))
			.unwrap(),
		)
	}
}

impl Display for Delta {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
