use axum::response::{IntoResponse, Redirect, Response};
use rocksdb::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WkError {
	#[error("Authentication required.")]
	Auth, // EPERM
	#[error("Permission denied.")]
	Access, // EACCES
	#[error("Invalid record.")]
	Corrupt, // EBADF
	#[error("Resource unavailable, try again.")]
	Io, // EIO
	#[error("Invalid argument.")]
	InvalidArgument, // EINVAL
}

impl IntoResponse for WkError {
	fn into_response(self) -> Response {
		match self {
			Self::Auth => {
				Redirect::to(&format!("/login?error={self}")).into_response()
			}
			Self::Access | Self::Corrupt | Self::Io | Self::InvalidArgument => {
				Redirect::to(&format!("?error={self}")).into_response()
			}
		}
	}
}

impl From<rocksdb::Error> for WkError {
	fn from(_: Error) -> Self {
		WkError::Io
	}
}

#[macro_export]
macro_rules! resource_or_return_error {
	($e:expr) => {
		match $e {
			Ok(Some(resource)) => resource,
			Ok(_) => return crate::not_found().await.into_response(),
			Err(e) => return e.into_response(),
		}
	};
}
