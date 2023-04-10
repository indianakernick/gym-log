use std::ops::ControlFlow;
use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn delete(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();
    let exercise_id = params.first("exerciseId").unwrap();

    if !common::is_uuid(workout_id) || !common::is_uuid(exercise_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    let client_version = match common::parse_request_json::<common::VersionDeleteReq>(&req) {
        Ok(b) => b.version,
        Err(r) => return r,
    };
    let collection_prefix = common::get_collection_prefix(
        common::collection_from_version(client_version)
    );

    common::version_apply(
        &req,
        client_version,
        |mut builder, user_id, new_version| {
            builder = common::check_exists(
                builder,
                user_id.clone(),
                format!("{collection_prefix}WORKOUT#{workout_id}"),
            );

            common::version_delete_item(
                builder,
                user_id,
                format!("{collection_prefix}WORKOUT#{workout_id}#{exercise_id}"),
                new_version,
            )
        },
        |reasons| {
            if reasons.iter().any(|r| r.code() == Some("ConditionalCheckFailed")) {
                ControlFlow::Break(common::empty_response(StatusCode::NOT_FOUND))
            } else {
                ControlFlow::Continue(())
            }
        }
    ).await
}

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();
    let exercise_id = params.first("exerciseId").unwrap();

    if let Err(e) = common::validate_uuid(workout_id) {
        return e;
    }

    if let Err(e) = common::validate_uuid(exercise_id) {
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
        |mut builder, user_id, new_version| {
            builder = common::check_exists(
                builder,
                user_id.clone(),
                common::make_key_from_id::<common::Workout>(&collection_prefix, workout_id),
            );

            common::version_put_item::<common::Exercise>(
                &format!("{workout_id}#{exercise_id}")
            )(builder, body.item, user_id, new_version)
        },
        |reasons| {
            if reasons[0].code() == Some("ConditionalCheckFailed") {
                return ControlFlow::Break(common::empty_response(StatusCode::NOT_FOUND));
            }

            ControlFlow::Continue(())
        }
    ).await
}
