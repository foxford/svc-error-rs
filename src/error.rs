use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error, fmt};

/// Configure and build an error.
#[derive(Debug)]
pub struct Builder {
    kind: Option<(String, String)>,
    detail: Option<String>,
    status: Option<StatusCode>,
}

/// Error object.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    #[serde(rename = "type")]
    kind: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[cfg(not(feature = "serialize-status-code"))]
    #[serde(skip)]
    status: StatusCode,
    #[cfg(feature = "serialize-status-code")]
    #[serde(with = "http_serde::status_code")]
    status: StatusCode,
    #[serde(skip)]
    extras: HashMap<String, String>,
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
            extras: HashMap::new(),
            status,
        }
    }

    /// Set kind and title of the error.
    pub fn set_kind(&mut self, kind: &str, title: &str) -> &mut Self {
        self.kind = kind.to_owned();
        self.title = title.to_owned();
        self
    }

    /// Return a kind for this error.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Return a title for this error.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set a status code information about the error.
    pub fn set_status_code(&mut self, value: StatusCode) -> &mut Self {
        self.status = value;
        self
    }

    /// Return a status code for this error.
    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    /// Return a detail for this error.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Set detailed information about the error.
    pub fn set_detail(&mut self, value: &str) -> &mut Self {
        self.detail = Some(value.to_owned());
        self
    }

    /// Return all extras for this error.
    pub fn extras(&self) -> &HashMap<String, String> {
        &self.extras
    }

    /// Set detailed information about the error.
    pub fn set_extra(&mut self, key: &str, value: &str) -> &mut Self {
        self.extras.insert(key.to_owned(), value.to_owned());
        self
    }

    /// Create an error builder object.
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[{}] {}", self.kind, self.title)?;

        if let Some(ref detail) = self.detail {
            write!(fmt, ": {detail}")?;
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
            extras: HashMap::new(),
            status,
        }
    }
}
