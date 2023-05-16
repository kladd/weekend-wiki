use std::sync::Arc;

use axum::{
	extract::State,
	response::{Html, IntoResponse, Redirect},
	Form,
};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;

use crate::{
	auth::{
		add_user_to_namespace, namespace::Namespace, user::User, COOKIE_NAME,
	},
	resource_or_return_error, Context, CONTROL_HTML,
};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ControlParams {
	CreateUser { username: String, password: String },
	AddUserToNamespace { username: String, namespace: String },
	SetNamespaceMode { namespace: String, mode: String },
}

pub async fn get(
	TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
	if let Some(_username) = cookies.get(COOKIE_NAME).filter(|username| {
		// TODO: Also, extremely secure.
		*username == User::META
	}) {
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
	if let Some(_username) = cookies.get(COOKIE_NAME).filter(|username| {
		// TODO: Also, extremely secure.
		*username == User::META
	}) {
		match params {
			ControlParams::CreateUser { username, password } => {
				// TODO: Check exists
				let mut user = User::new(&username, &password);
				// TODO: Check exists
				let mut ns = Namespace::new(&username, &username, 0o700);
				// TODO: Meta.
				add_user_to_namespace(&state.db, &mut user, &mut ns).await;
				println!("added {user:?} to {ns:?}");
				Redirect::to("/control?success=YES").into_response()
			}
			ControlParams::AddUserToNamespace {
				username,
				namespace,
			} => {
				let user = User::get(&state.db, &username).await;
				let ns = resource_or_return_error!(
					Namespace::get(&state.db, &namespace).await
				);

				if let (Some(mut user), mut ns) = (user, ns) {
					add_user_to_namespace(&state.db, &mut user, &mut ns).await;
					println!("added {user:?} to {ns:?}");
					Redirect::to("/control?success=YES").into_response()
				} else {
					Redirect::to("/control?error=ENOENT").into_response()
				}
			}
			ControlParams::SetNamespaceMode { namespace, mode } => {
				let mut ns = resource_or_return_error!(
					Namespace::get(&state.db, &namespace).await
				);
				// TODO: validate input obviously
				ns.mode = u16::from_str_radix(&mode, 8).unwrap();
				Namespace::put(&state.db, &ns).await;
				dbg!(ns);
				Redirect::to("/control?success=YES").into_response()
			}
		}
	} else {
		Redirect::to("/?error=EPERM").into_response()
	}
}
