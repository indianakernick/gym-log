use lambda_http::{Request, Response, Error, Body};
use serde::Deserialize;
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
        .header("Access-Control-Allow-Headers", "Authorization")
}

pub async fn options(_req: Request) -> Result {
    Ok(with_cors(Response::builder())
        .status(200)
        .body(().into())
        .map_err(Box::new)?)
}
