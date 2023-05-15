use slug::slugify;

#[derive(Debug)]
pub struct DocumentKey(pub String, pub String);

#[derive(bincode::Encode, bincode::Decode)]
pub struct Document {
	title: String,
	slug: String,
	mode: u16,
	content: Option<String>,
}

impl Document {
	// TODO: Better signature.
	pub fn new(title: String, mode: u16, content: Option<String>) -> Self {
		Self {
			mode,
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
}

impl DocumentKey {
	pub fn from_bytes<B>(bytes: B) -> Self
	where
		B: AsRef<[u8]>,
	{
		// TODO: Decoding errors.
		let key_str = String::from_utf8(bytes.as_ref().to_vec()).unwrap();
		let (ns, slug) = key_str.split_once('/').unwrap();
		Self(ns.to_string(), slug.to_string())
	}
}
