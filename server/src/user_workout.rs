use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Request, RequestExt, http::StatusCode};
use super::{common, model};

pub async fn delete(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if !common::is_uuid(workout_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    let db = common::get_db_client().await;

    let result = db.delete_item()
        .table_name(common::TABLE_USER_SET)
        .key("UserId", AttributeValue::S(user_id))
        .key("Id", AttributeValue::S(workout_id.into()))
        .condition_expression("attribute_exists(UserId)")
        .send()
        .await;

    common::delete_response(result)
}

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if let Err(e) = common::validate_uuid(workout_id) {
        return e;
    }

    let workout = match common::parse_request_json::<model::Workout>(&req) {
        Ok(w) => w,
        Err(e) => return e,
    };

    if let Some(dt) = workout.start_time {
        if let Err(e) = chrono::NaiveDateTime::parse_from_str(dt, "%FT%TZ") {
            return common::error_response(StatusCode::BAD_REQUEST, &format!("Invalid start_time: {}", e));
        }
    }

    if let Some(dt) = workout.finish_time {
        if let Err(e) = chrono::NaiveDateTime::parse_from_str(dt, "%FT%TZ") {
            return common::error_response(StatusCode::BAD_REQUEST, &format!("Invalid finish_time: {}", e));
        }
    }

    let db = common::get_db_client().await;

    let mut builder = db.put_item()
        .table_name(common::TABLE_USER_SET)
        .item("UserId", AttributeValue::S(user_id))
        .item("Id", AttributeValue::S(workout_id.into()))
        .item("WorkoutNotes", AttributeValue::S(workout.notes.into()));

    if let Some(dt) = workout.start_time {
        builder = builder.item("StartTime", AttributeValue::S(dt.into()));
    }

    if let Some(dt) = workout.finish_time {
        builder = builder.item("FinishTime", AttributeValue::S(dt.into()));
    }

    builder.send().await?;

    common::empty_response(StatusCode::OK)
}
