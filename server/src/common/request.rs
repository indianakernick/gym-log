use base64::Engine;
use lambda_http::{Request, http::StatusCode};
use serde::Deserialize;

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

pub fn parse_request_json<'de, T: Deserialize<'de>>(
    req: &'de Request,
) -> Result<T, super::Result> {
    match serde_json::from_slice::<T>(req.body().as_ref()) {
        Ok(t) => Ok(t),
        Err(e) => Err(super::error_response(StatusCode::BAD_REQUEST, &e.to_string()))
    }
}

pub fn is_uuid(id: &str) -> bool {
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
            14 => {
                if b != b'4' {
                    return false;
                }
            }
            19 => {
                if ![b'8', b'9', b'a', b'b', b'A', b'B'].contains(&b) {
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

    true
}

pub fn validate_uuid(id: &str) -> Result<(), super::Result> {
    if is_uuid(id) {
        Ok(())
    } else {
        Err(super::error_response(StatusCode::BAD_REQUEST, "invalid UUID in path"))
    }
}

pub fn is_date(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%F").is_ok()
}

pub fn validate_date(date: &str) -> Result<(), super::Result> {
    if is_date(date) {
        Ok(())
    } else {
        Err(super::error_response(StatusCode::BAD_REQUEST, "invalid date in path"))
    }
}
