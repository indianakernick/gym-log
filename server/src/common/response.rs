use lambda_http::{Response, Body, Error, http::StatusCode};
use serde::Serialize;

pub type Result = std::result::Result<Response<Body>, Error>;

pub fn empty_response(status: StatusCode) -> Result {
    Response::builder()
        .status(status)
        .body(().into())
        .map_err(|e| e.into())
}

pub fn json_response<T: Serialize>(status: StatusCode, value: T) -> Result {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&value).unwrap().into())
        .map_err(|e| e.into())
}

pub fn error_response(status: StatusCode, message: &str) -> Result {
    #[derive(Serialize)]
    struct Error<'a> {
        message: &'a str,
    }

    json_response(status, Error { message })
}
