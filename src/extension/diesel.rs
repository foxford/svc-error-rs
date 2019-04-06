use diesel::result::Error;
use http::StatusCode;

use crate::Error as TheError;

impl From<Error> for TheError {
    fn from(value: Error) -> TheError {
        let status = match &value {
            Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        let mut err = TheError::from(status);
        err.set_detail(&format!("a database error = '{}'", &value));
        err
    }
}
