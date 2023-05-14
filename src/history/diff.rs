use bincode::{Decode, Encode};

const DIFF_CONTEXT: usize = 3;

#[derive(Encode, Decode, Debug)]
pub struct Delta {
	pub(super) patch: String,
}

impl Delta {
	pub fn new<L, R>(slug: &str, prev: L, next: R) -> Self
	where
		L: AsRef<str>,
		R: AsRef<str>,
	{
		Self {
			patch: String::from_utf8(unified_diff::diff(
				prev.as_ref().as_bytes(),
				slug,
				next.as_ref().as_bytes(),
				slug,
				DIFF_CONTEXT,
			))
			.unwrap(),
		}
	}
}
