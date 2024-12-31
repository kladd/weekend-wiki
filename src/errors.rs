use axum::response::{IntoResponse, Redirect, Response};
use rocksdb::Error;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
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

/// Unwraps a `Result<T>` or returns an HTTP error response.
#[macro_export]
macro_rules! ok {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(e) => return e.into_response(),
		}
	};
}

/// Unwraps an `Option<T>` or returns an HTTP not found response.
#[macro_export]
macro_rules! exists {
	($e:expr) => {
		match $e {
			Some(resource) => resource,
			None => return $crate::not_found().await.into_response(),
		}
	};
}
