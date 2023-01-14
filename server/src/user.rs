use lambda_http::{Request, Response};
use super::common;

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);

    Ok(common::with_cors(Response::builder())
        .status(200)
        .header("Content-Type", "text/plain")
        .body(user_id.into())
        .map_err(Box::new)?)
}
