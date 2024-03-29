use askama::Template;

use crate::{
	auth::user::UserView,
	history::{
		db::{HistoryKey, HistoryRecord},
		delta::Delta,
	},
};

#[derive(Template)]
#[template(path = "history.html")]
pub struct HistoryView<'a> {
	pub slug: &'a str,
	pub title: &'a str,
	pub revisions: Vec<HistoryRevisionView>,
	pub user: Option<UserView>,
}

pub struct HistoryRevisionView {
	version: String,
	delta: Delta,
}

impl From<(HistoryKey, HistoryRecord)> for HistoryRevisionView {
	fn from((key, record): (HistoryKey, HistoryRecord)) -> Self {
		Self {
			version: key.revision(),
			delta: record.delta,
		}
	}
}
