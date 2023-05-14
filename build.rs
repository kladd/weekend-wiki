use std::{env, fs, fs::File, io::Write, path::Path};

use quote::quote;

const CONFIG_FILE: &str = "config.rs";

fn main() {
	let mut file = File::create(
		Path::new(&env::var("OUT_DIR").unwrap()).join(CONFIG_FILE),
	)
	.unwrap();

	// TODO: Auto-gen from each file in static if this gets annoying.
	let index_html = fs::read_to_string("static/index.html").unwrap();
	let create_html = fs::read_to_string("static/create.html").unwrap();

	let config = quote! {
		const LOCAL_DB_PATH: &str = concat!(env!("OUT_DIR"), "/wiki.db");

		const INDEX_HTML: &str = #index_html;
		const CREATE_HTML: &str = #create_html;
	};

	writeln!(file, "{config}").unwrap();
}
