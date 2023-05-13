use slug::slugify;

use crate::BINCODE_CONFIG;

#[derive(bincode::Encode, bincode::Decode)]
pub struct Document {
	title: String,
	slug: String,
	content: Option<String>,
}

impl Document {
	pub fn new(title: String, content: Option<String>) -> Self {
		Self {
			title: title.clone(),
			// TODO: I don't really like that this doesn't retain
			//       capitalization. Write a slugger like Wikipedia, e.g.
			//       https://en.wikipedia.org/wiki/Clean_URL
			slug: slugify(title),
			content,
		}
	}

	pub fn slug(&self) -> &String {
		&self.slug
	}

	pub fn title(&self) -> &String {
		&self.title
	}

	pub fn content(&self) -> Option<&String> {
		self.content.as_ref()
	}

	pub fn set_content(&mut self, content: String) {
		self.content = Some(content)
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		// TODO: Encoding errors.
		bincode::encode_to_vec(self, BINCODE_CONFIG).unwrap()
	}

	pub fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		// TODO: Decoding errors.
		let (doc, _) =
			bincode::decode_from_slice(bytes.as_ref(), BINCODE_CONFIG).unwrap();
		doc
	}
}
