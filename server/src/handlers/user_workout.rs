use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn delete(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if !common::is_uuid(workout_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    common::version_delete::<common::Workout>(&req, workout_id).await
}

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if let Err(e) = common::validate_uuid(workout_id) {
        return e;
    }

    common::version_modify(
        &req,
        common::version_put_item::<common::Workout>(workout_id)
    ).await
}
