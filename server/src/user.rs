use lambda_http::{Request, Response, Body, Error};
use super::common;

pub async fn get(event: Request) -> Result<Response<Body>, Error> {
    let user_id = common::get_user_id(event);

    Ok(common::with_cors(Response::builder())
        .status(200)
        .header("Content-Type", "text/plain")
        .body(user_id.into())
        .map_err(Box::new)?)
}
