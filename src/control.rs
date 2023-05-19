use std::sync::Arc;

use axum::{
	extract::State,
	response::{Html, IntoResponse, Redirect},
	Form,
};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;

use crate::{
	auth::{add_user_to_namespace, namespace::Namespace, user::User},
	exists, ok,
	page::Page,
	Context, CONTROL_HTML,
};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ControlParams {
	CreateUser {
		username: String,
		password: String,
	},
	AddUserToNamespace {
		username: String,
		namespace: String,
	},
	SetPageMode {
		namespace: String,
		slug: String,
		mode: String,
	},
	SetNamespaceMode {
		namespace: String,
		mode: String,
	},
}

pub async fn get(
	State(state): State<Arc<Context>>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
	if ok!(User::authenticated(&state.db, cookies).await)
		.filter(|user| user.name == User::META)
		.is_some()
	{
		Html(CONTROL_HTML).into_response()
	} else {
		Redirect::to("/?error=EPERM").into_response()
	}
}

#[axum_macros::debug_handler]
pub async fn post(
	State(state): State<Arc<Context>>,
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
	Form(params): Form<ControlParams>,
) -> impl IntoResponse {
	if ok!(User::authenticated(&state.db, cookies).await)
		.filter(|user| user.name == User::META)
		.is_some()
	{
		match params {
			ControlParams::CreateUser { username, password } => {
				// TODO: Check exists
				let mut user = User::new(&username, &password);
				// TODO: Check exists
				let mut ns = Namespace::new(&username, &username, 0o700);
				// TODO: Meta.
				ok!(add_user_to_namespace(&state.db, &mut user, &mut ns).await);
				println!("added {user:?} to {ns:?}");
				Redirect::to("/control?success=YES").into_response()
			}
			ControlParams::AddUserToNamespace {
				username,
				namespace,
			} => {
				let user = User::get(&state.db, &username).await;
				let ns =
					exists!(ok!(Namespace::get(&state.db, &namespace).await));

				if let (Some(mut user), mut ns) = (user, ns) {
					ok!(add_user_to_namespace(&state.db, &mut user, &mut ns)
						.await);
					println!("added {user:?} to {ns:?}");
					Redirect::to("/control?success=YES").into_response()
				} else {
					Redirect::to("/control?error=ENOENT").into_response()
				}
			}
			ControlParams::SetNamespaceMode { namespace, mode } => {
				let mut ns =
					exists!(ok!(Namespace::get(&state.db, &namespace).await));
				// TODO: validate input obviously
				ns.mode = u16::from_str_radix(&mode, 8).unwrap();
				if let Err(e) = Namespace::put(&state.db, &ns).await {
					return e.into_response();
				}
				dbg!(ns);
				Redirect::to("/control?success=YES").into_response()
			}
			ControlParams::SetPageMode {
				namespace,
				slug,
				mode,
			} => {
				let mut page =
					exists!(Page::get(&state.db, &namespace, &slug).await);
				page.mode = u16::from_str_radix(&mode, 8).unwrap();
				Page::put(&state.db, &namespace, &page).await;
				dbg!(page);

				Redirect::to("/control?success=YES").into_response()
			}
		}
	} else {
		Redirect::to("/?error=EPERM").into_response()
	}
}
