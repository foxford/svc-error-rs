use http::StatusCode;
use svc_agent::Error;

use crate::Error as TheError;

impl From<Error> for TheError {
    fn from(value: Error) -> TheError {
        let mut err = TheError::from(StatusCode::UNPROCESSABLE_ENTITY);
        err.set_detail(&value.to_string());
        err
    }
}
