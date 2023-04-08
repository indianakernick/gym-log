use std::ops::ControlFlow;
use aws_sdk_dynamodb::types::AttributeValue;
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

    let body = match common::parse_request_json::<common::VersionModifyReq<_>>(&req) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let collection_prefix = common::get_collection_prefix(
        common::collection_from_version(body.version)
    );

    common::version_apply(
        &req,
        body.version,
        |builder, user_id, new_version| {
            common::version_put_item(
                format!("{collection_prefix}WORKOUT#{workout_id}"),
                |mut builder, workout: common::Workout| {
                    builder = builder.item("Notes", AttributeValue::S(workout.notes.0.into()));

                    if let Some(dt) = workout.start_time {
                        builder = builder.item("StartTime", AttributeValue::S(dt.into()));
                    }

                    if let Some(dt) = workout.finish_time {
                        builder = builder.item("FinishTime", AttributeValue::S(dt.into()));
                    }

                    builder
                }
            )(builder, body.item, user_id, new_version)
        },
        |_| ControlFlow::Continue(()),
    ).await
}
