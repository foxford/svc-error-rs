#![deny(missing_debug_implementations, missing_docs)]

//! SvcError is an implementation of RFC7807: Problem Details for HTTP APIs.

use http::StatusCode;
use serde_derive::Serialize;
use std::{error, fmt};

/// Configure and build an error.
#[derive(Debug)]
pub struct Builder {
    kind: Option<(String, String)>,
    detail: Option<String>,
    status: Option<StatusCode>,
}

/// Error object.
#[derive(Debug, Serialize)]
pub struct Error {
    #[serde(rename = "type")]
    kind: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip)]
    status: StatusCode,
}

impl Builder {
    fn new() -> Self {
        Self {
            kind: None,
            detail: None,
            status: None,
        }
    }

    /// Set status of the error.
    pub fn status(self, status: StatusCode) -> Self {
        Self {
            status: Some(status),
            ..self
        }
    }

    /// Set kind and title of the error.
    pub fn kind(self, kind: &str, title: &str) -> Self {
        Self {
            kind: Some((kind.to_owned(), title.to_owned())),
            ..self
        }
    }

    /// Set detailed information about the error.
    pub fn detail(self, detail: &str) -> Self {
        Self {
            detail: Some(detail.to_owned()),
            ..self
        }
    }

    /// Create an error object.
    pub fn build(self) -> Error {
        let mut err = match (self.kind, self.status) {
            (Some((ref kind, ref title)), Some(status)) => Error::new(kind, title, status),
            (None, Some(status)) => Error::from(status),
            _ => Error::from(StatusCode::INTERNAL_SERVER_ERROR),
        };

        match self.detail {
            Some(ref detail) => {
                err.set_detail(detail);
                err
            }
            None => err,
        }
    }
}

impl Error {
    /// Create an error object.
    pub fn new(kind: &str, title: &str, status: StatusCode) -> Self {
        Self {
            kind: kind.to_owned(),
            title: title.to_owned(),
            detail: None,
            status,
        }
    }

    /// Set detailed information about the error.
    pub fn set_detail(&mut self, value: &str) -> &mut Self {
        self.detail = Some(value.to_owned());
        self
    }

    /// Return a status code for this error.
    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    /// Create an error builder object.
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.status_code()
            .canonical_reason()
            .unwrap_or("Unknown status code")
    }
}

impl fmt::Display for Error {
    #[allow(deprecated)] // .cause() is deprecated on nightly
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[{}] {}", self.kind, self.title,)?;

        if let Some(ref detail) = self.detail {
            write!(fmt, ": {}", detail)?;
        }

        use std::error::Error;
        if let Some(ref cause) = self.cause() {
            write!(fmt, ": {}", cause)?;
        }

        Ok(())
    }
}

impl From<StatusCode> for Error {
    fn from(status: StatusCode) -> Self {
        let title = status.canonical_reason().unwrap_or("Unknown status code");
        Self {
            kind: String::from("about:blank"),
            title: title.to_owned(),
            detail: None,
            status,
        }
    }
}
