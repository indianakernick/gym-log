use lambda_http::{Response, Body, Error, http::{response::Builder, StatusCode}};
use serde::Serialize;

pub type Result = std::result::Result<Response<Body>, Error>;

pub fn with_cors(builder: Builder) -> Builder {
    // We'll end up saying that we allow more methods than we actually do but I
    // don't think that matters much.

    builder
        // TODO: don't forgot to remove the *
        // This environment variable isn't going to exist until the
        // infrastructure is deployed. The Lambda must be compiled before the
        // infrastructure can be deployed. So I guess some manual intervention
        // would be required when deploying for the first time.
        // .header("Access-Control-Allow-Origin", dotenv_codegen::dotenv!("CFN_WebsiteUrl"))
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "OPTIONS,PUT,GET,DELETE")
        .header("Access-Control-Allow-Headers", "Authorization,Content-Type")
        .header("Access-Control-Max-Age", "86400")
}

pub fn empty_response(status: StatusCode) -> Result {
    with_cors(Response::builder())
        .status(status)
        .body(().into())
        .map_err(|e| e.into())
}

pub fn json_response<T: Serialize>(status: StatusCode, value: T) -> Result {
    with_cors(Response::builder())
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
