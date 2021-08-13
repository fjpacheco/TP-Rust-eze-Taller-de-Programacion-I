use crate::server_html::status_codes::status_code::StatusCode;
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq)]

/// HttpError defines an error to be displayed as a reply
/// to an interrupted request.
pub struct HttpError {
    status_code: StatusCode,
}

impl HttpError {
    pub fn new(status_code: StatusCode) -> HttpError {
        HttpError { status_code }
    }

    /// # Return value
    /// Returns the code and description (as a tuple) of the contained error
    pub fn take(self) -> (String, String) {
        let mut code = self.status_code.to_string();
        let mut description = code.split_off(3);
        description.remove(0);
        (code, description)
    }

    pub fn get_status_code(&self) -> StatusCode {
        self.status_code.clone()
    }
}

impl From<StatusCode> for HttpError {
    /// Creates an HttpError from a give status code.
    fn from(status_code: StatusCode) -> HttpError {
        HttpError { status_code }
    }
}