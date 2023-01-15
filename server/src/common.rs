use lambda_http::{Request, Response, Error, Body, http::StatusCode};
use serde::{Deserialize, Serialize};
use base64::Engine;
use lambda_http::http::response::Builder;

pub const TABLE_USER_MEASUREMENT: &str = "gym-log.UserMeasurement";
pub const TABLE_USER_SET: &str = "gym-log.UserSet";

pub type Result = std::result::Result<Response<Body>, Error>;

pub async fn get_db_client() -> aws_sdk_dynamodb::Client {
    let config = aws_config::load_from_env().await;
    aws_sdk_dynamodb::Client::new(&config)
}

pub fn get_user_id(req: &Request) -> String {
    // API Gateway validates that the header exists and the JWT within has a
    // valid signature. We can safely assume that the JWT is completely valid.

    #[derive(Deserialize)]
    struct Claims {
        sub: String,
    }

    let engine = base64::engine::general_purpose::URL_SAFE;

    let access_token = req.headers()["Authorization"].as_bytes();
    let mut parts = access_token.split(|c| *c == b'.');

    parts.next();

    let claims_b64 = parts.next().unwrap();
    let claims_bytes = engine.decode(claims_b64).unwrap();
    let claims: Claims = serde_json::from_slice(&claims_bytes).unwrap();

    claims.sub
}

pub fn with_cors(builder: Builder) -> Builder {
    // We'll end up saying that we allow more methods than we actually do but I
    // don't think that matters much.

    builder
        .header("Access-Control-Allow-Origin", "http://gymlog.indianakernick.com.s3-website-ap-southeast-2.amazonaws.com")
        .header("Access-Control-Allow-Methods", "OPTIONS,PUT,GET,DELETE")
        .header("Access-Control-Allow-Headers", "Authorization,Content-Type")
}

pub fn options() -> Result {
    empty_response(StatusCode::OK)
}

pub fn empty_response(status: StatusCode) -> Result {
    with_cors(Response::builder())
        .status(status)
        .body(().into())
        .map_err(|e| e.into())
}

pub fn error_response(status: StatusCode, message: &str) -> Result {
    #[derive(Serialize)]
    struct Error<'a> {
        message: &'a str,
    }

    with_cors(Response::builder())
        .status(status)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&Error { message }).unwrap().into())
        .map_err(|e| e.into())
}

pub fn parse_request_json<'de, T: serde::Deserialize<'de>>(
    req: &'de Request,
) -> std::result::Result<T, Result> {
    match serde_json::from_slice::<T>(req.body().as_ref()) {
        Ok(t) => Ok(t),
        Err(e) => Err(error_response(StatusCode::BAD_REQUEST, &e.to_string()))
    }
}

fn is_uuid(id: &str) -> bool {
    if id.len() != 36 {
        return false;
    }

    for (i, b) in id.bytes().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if b != b'-' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }

    return true;
}

pub fn validate_uuid(id: &str) -> std::result::Result<(), Result> {
    if is_uuid(id) {
        Ok(())
    } else {
        Err(error_response(StatusCode::BAD_REQUEST, "ID must be UUID"))
    }
}
