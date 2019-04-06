use http::StatusCode;
use svc_authn::{Error, SerializationError};

use crate::Error as TheError;

impl From<Error> for TheError {
    fn from(value: Error) -> TheError {
        let mut err = TheError::from(StatusCode::UNAUTHORIZED);
        err.set_detail(&value.to_string());
        err
    }
}

impl From<SerializationError> for TheError {
    fn from(value: SerializationError) -> TheError {
        let mut err = TheError::from(StatusCode::UNPROCESSABLE_ENTITY);
        err.set_detail(&format!("serialization error: {}", &value));
        err
    }
}
