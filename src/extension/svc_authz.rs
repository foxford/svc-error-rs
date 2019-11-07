use http::StatusCode;
use svc_authz::error::{Error, IntentError, Kind};

use crate::Error as TheError;

impl From<Error> for TheError {
    fn from(value: Error) -> TheError {
        let fun = |status: StatusCode, inner: &IntentError| -> TheError {
            let mut err = TheError::from(status);
            err.set_detail(&inner.to_string());
            err
        };
        match value.kind() {
            Kind::Forbidden(ref inner) => fun(StatusCode::FORBIDDEN, inner),
            Kind::Network(ref inner) => fun(StatusCode::FAILED_DEPENDENCY, inner),
            Kind::Internal(ref inner) => fun(StatusCode::UNPROCESSABLE_ENTITY, inner),
        }
    }
}
