use std::{env, fs, fs::File, io::Write, path::Path};

use quote::quote;

const CONFIG_FILE: &str = "config.rs";

fn main() {
	let mut file = File::create(
		Path::new(&env::var("OUT_DIR").unwrap()).join(CONFIG_FILE),
	)
	.unwrap();

	// TODO: Auto-gen from each file in static if this gets annoying.
	let create_html = fs::read_to_string("static/create.html").unwrap();
	let login_html = fs::read_to_string("static/login.html").unwrap();
	let ctrl_html = fs::read_to_string("static/control.html").unwrap();

	let config = quote! {
		const LOCAL_DB_PATH: &str = concat!(env!("OUT_DIR"), "/wiki.db");

		const CREATE_HTML: &str = #create_html;
		const LOGIN_HTML: &str = #login_html;
		const CONTROL_HTML: &str = #ctrl_html;
	};

	writeln!(file, "{config}").unwrap();
}
