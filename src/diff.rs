use std::collections::BTreeMap;

use ::diff as libdiff;
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub struct Delta {
	changes: BTreeMap<usize, Action>,
}

#[derive(Encode, Decode, Debug)]
pub enum Action {
	Remove(String),
	Add(String),
}

impl Delta {
	pub fn new<L, R>(prev: L, next: R) -> Self
	where
		L: AsRef<str>,
		R: AsRef<str>,
	{
		let mut changes = BTreeMap::new();
		let diff = libdiff::lines(prev.as_ref(), next.as_ref());

		// TODO: Context for human viewing. Keep a ring buffer or something.
		for (line, change) in diff.iter().enumerate() {
			match change {
				libdiff::Result::Left(l) => {
					changes.insert(line, Action::Remove(l.to_string()));
				}
				libdiff::Result::Right(r) => {
					changes.insert(line, Action::Add(r.to_string()));
				}
				_ => (),
			}
		}

		Self { changes }
	}

	pub fn apply<S>(&self, text: S) -> String
	where
		S: AsRef<str>,
	{
		self.apply_(text, false)
	}

	pub fn revert<S>(&self, text: S) -> String
	where
		S: AsRef<str>,
	{
		self.apply_(text, true)
	}

	// TODO: This is probably very stupid to anyone that knows what they're
	//       doing.
	fn apply_<S>(&self, text: S, revert: bool) -> String
	where
		S: AsRef<str>,
	{
		let lines = text.as_ref().lines().enumerate();

		let mut changes = self.changes.iter().peekable();
		let mut next_change = changes.next();

		let mut line_buffer = lines
			.filter_map(|(line, text)| {
				match next_change {
					Some((line_no, action)) if *line_no == line => {
						next_change = changes.next();
						// emit line or remove or something.
						match action {
							Action::Add(_) if revert => None,
							// CLion Rust plugin bug? It thinks `revert` is used
							// after move.
							Action::Remove(r) if revert => Some(r.as_ref()),
							Action::Add(l) => Some(l.as_ref()),
							Action::Remove(_) => None,
						}
					}
					_ => Some(text),
				}
			})
			.collect::<Vec<_>>();

		// Don't lose a line break if we're continuing beyond the end of the
		// original file.
		if changes.peek().is_some() {
			line_buffer.push("");
		}

		line_buffer.append(
			&mut changes
				.map(|(_, action)| {
					// All remaining actions should be adds in either apply
					// or revert mode.
					match action {
						Action::Add(s) | Action::Remove(s) => s.as_ref(),
					}
				})
				.collect::<Vec<_>>(),
		);
		line_buffer.join("\n")
	}
}

#[cfg(test)]
mod tests {
	use crate::diff::Delta;

	#[test]
	#[ignore]
	fn diff() {
		let a = r#"
Decimal is a numeral system used by humans for everyday counting and arithmetic operations. It is also known as the base-10 system because it uses ten digits: 0, 1, 2, 3, 4, 5, 6, 7, 8, and 9. The word "decimal" itself comes from the Latin word "decem," meaning "ten."

In the decimal system, numbers are represented using place value, where the position of each digit determines its value. The rightmost digit represents the ones place, the digit to the left represents the tens place, the next digit represents the hundreds place, and so on. Each digit is multiplied by the corresponding power of ten based on its position.

For example, the number 325 in decimal represents 3 * 10^2 + 2 * 10^1 + 5 * 10^0, which is equal to 300 + 20 + 5 = 325.

Decimal notation is commonly used in everyday life for various purposes, such as counting, measuring, and representing quantities. It is the most familiar and widely used numeral system by people around the world, making it an essential part of mathematics and everyday calculations.	
		"#.trim();
		let b = r#"
Decimal is a numeral system used by humans for everyday counting and arithmetic operations. It is also known as the base-10 system because it uses ten digits: 0, 1, 2, 3, 4, 5, 6, 7, 8, and 9. The word "decimal" itself comes from the Latin word "decem," meaning "ten."

For example, the number 325 in decimal represents 3 * 10^2 + 2 * 10^1 + 5 * 10^0, which is equal to 300 + 20 + 5 = 325.

New line.

Decimal notation is commonly used, for counting, measuring, and representing quantities. It is the most familiar and widely used numeral system by people around the world, making it an essential part of mathematics and everyday calculations.	
		"#.trim();

		let delta = Delta::new(a, b);

		dbg!(&delta);

		assert_eq!(b, delta.apply(a));
		// assert_eq!(a, delta.revert(b));
	}
}
