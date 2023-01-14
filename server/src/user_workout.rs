use lambda_http::{Request, RequestExt, Response};
use super::common;

pub async fn delete(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    Ok(common::with_cors(Response::builder())
        .status(200)
        .header("Content-Type", "text/plain")
        .body((String::from(user_id) + workout_id).into())
        .map_err(Box::new)?)
}

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    Ok(common::with_cors(Response::builder())
        .status(200)
        .header("Content-Type", "text/plain")
        .body((String::from(user_id) + workout_id).into())
        .map_err(Box::new)?)
}
