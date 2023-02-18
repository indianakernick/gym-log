use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn delete(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if !common::is_uuid(workout_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    common::version_delete(&req, format!("WORKOUT#{workout_id}")).await
}

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if let Err(e) = common::validate_uuid(workout_id) {
        return e;
    }

    common::version_modify(&req, common::version_put_item(
        format!("WORKOUT#{workout_id}"),
        |mut builder, item: common::Workout| {
            builder = builder.item("Notes", AttributeValue::S(item.notes.0.into()));

            if let Some(dt) = item.start_time {
                builder = builder.item("StartTime", AttributeValue::S(dt.into()));
            }

            if let Some(dt) = item.finish_time {
                builder = builder.item("FinishTime", AttributeValue::S(dt.into()));
            }

            builder
        }
    )).await
}
