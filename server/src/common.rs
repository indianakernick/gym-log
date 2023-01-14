use lambda_http::{Request, Response, Error, Body};
use serde::Deserialize;
use base64::Engine;
use lambda_http::http::response::Builder;

pub fn get_user_id(req: Request) -> String {
    // API Gateway validates that the header exists and the JWT within has a
    // valid signature. We can safely assume that the JWT is completely valid.

    #[derive(Deserialize)]
    struct Claims {
        sub: String,
    }

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

    let access_token = req.headers()["Authorization"].as_bytes();
    let mut parts = access_token.split(|c| *c == b'.');

    parts.next();

    let claims_b64 = parts.next().unwrap();
    let claims_bytes = engine.decode(claims_b64).unwrap();
    let claims: Claims = serde_json::from_slice(&claims_bytes).unwrap();

    claims.sub
}

pub fn with_cors(builder: Builder) -> Builder {
    // We'll end up saying that we allow PUT on /user but I don't think it
    // matters.

    builder
        .header("Access-Control-Allow-Origin", "http://gymlog.indianakernick.com.s3-website-ap-southeast-2.amazonaws.com")
        .header("Access-Control-Allow-Methods", "OPTIONS,PUT,GET")
        .header("Access-Control-Allow-Headers", "Authorization")
}

pub async fn options(_event: Request) -> Result<Response<Body>, Error> {
    Ok(with_cors(Response::builder())
        .status(200)
        .body(().into())
        .map_err(Box::new)?)
}
