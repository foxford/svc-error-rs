use http::StatusCode;
use r2d2::Error;

use crate::Error as TheError;

impl From<Error> for TheError {
    fn from(value: Error) -> TheError {
        let mut err = TheError::from(StatusCode::UNPROCESSABLE_ENTITY);
        err.set_detail(&format!("a connection pool error = '{}'", &value));
        err
    }
}
