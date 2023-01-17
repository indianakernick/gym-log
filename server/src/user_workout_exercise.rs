use aws_sdk_dynamodb::{model::{AttributeValue, TransactWriteItem, Put, Delete}, types::SdkError, error::TransactWriteItemsErrorKind};
use lambda_http::{Request, RequestExt, http::StatusCode};
use super::{common, model};

pub async fn delete(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();
    let exercise_id = params.first("exerciseId").unwrap();

    if !common::is_uuid(workout_id) || !common::is_uuid(exercise_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    let db = common::get_db_client();

    let mut builder = db.transact_write_items()
        .transact_items(TransactWriteItem::builder()
            .delete(Delete::builder()
                .table_name(common::TABLE_USER_SET)
                .key("UserId", AttributeValue::S(user_id.clone()))
                .key("Id", AttributeValue::S(format!("{}#{}", workout_id, exercise_id)))
                .condition_expression("attribute_exists(UserId)")
                .build()
            )
            .build());

    let mut output = db.query()
        .table_name(common::TABLE_USER_SET)
        .key_condition_expression("UserId = :userId AND begins_with(Id, :id)")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .expression_attribute_values(":id", AttributeValue::S(format!("{}#{}#", workout_id, exercise_id)))
        .projection_expression("Id")
        .send()
        .await?;

    for mut item in output.items.take().unwrap().into_iter() {
        builder = builder.transact_items(TransactWriteItem::builder()
            .delete(Delete::builder()
                .table_name(common::TABLE_USER_SET)
                .key("UserId", AttributeValue::S(user_id.clone()))
                .key("Id", item.remove("Id").unwrap())
                .build()
            )
            .build());
    }

    let result = builder.send().await;

    // All this to check if the thing we're deleting existed before we tried to
    // delete it...
    if let Err(e) = result {
        if let SdkError::ServiceError(ref service_err) = e {
            if let TransactWriteItemsErrorKind::TransactionCanceledException(ref canceled) = service_err.err().kind {
                if let Some(reasons) = canceled.cancellation_reasons() {
                    if reasons.iter().any(|r| r.code() == Some("ConditionalCheckFailed")) {
                        return common::empty_response(StatusCode::NOT_FOUND);
                    }
                }
            }
        }
        return Err(e.into());
    }

    common::empty_response(StatusCode::OK)
}

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();
    let exercise_id = params.first("exerciseId").unwrap();

    if let Err(e) = common::validate_uuid(workout_id) {
        return e;
    }

    if let Err(e) = common::validate_uuid(exercise_id) {
        return e;
    }

    let exercise = match common::parse_request_json::<model::Exercise>(&req) {
        Ok(w) => w,
        Err(e) => return e,
    };

    for set in exercise.sets.iter() {
        if let Err(e) = common::validate_uuid(set.set_id) {
            return e;
        }
    }

    let db = common::get_db_client();

    let mut builder = db.transact_write_items()
        .transact_items(TransactWriteItem::builder()
            .put(Put::builder()
                .table_name(common::TABLE_USER_SET)
                .item("UserId", AttributeValue::S(user_id.clone()))
                .item("Id", AttributeValue::S(format!("{}#{}", workout_id, exercise_id)))
                .item("ExerciseOrder", AttributeValue::N(exercise.order.to_string()))
                .item("ExerciseType", AttributeValue::S(exercise.r#type.into()))
                .item("ExerciseNotes", AttributeValue::S(exercise.notes.into()))
                .build())
            .build());

    for set in exercise.sets.iter() {
        let mut put = Put::builder()
            .table_name(common::TABLE_USER_SET)
            .item("UserId", AttributeValue::S(user_id.clone()))
            .item("Id", AttributeValue::S(format!("{}#{}#{}", workout_id, exercise_id, set.set_id)))
            .item("SetOrder", AttributeValue::N(set.order.to_string()));

        if let Some(a) = set.repetitions {
            put = put.item("Repetitions", AttributeValue::N(a.to_string()));
        }

        if let Some(a) = set.resistance {
            put = put.item("Resistance", AttributeValue::N(a.to_string()));
        }

        if let Some(a) = set.speed {
            put = put.item("Speed", AttributeValue::N(a.to_string()));
        }

        if let Some(a) = set.distance {
            put = put.item("Distance", AttributeValue::N(a.to_string()));
        }

        if let Some(a) = set.duration {
            put = put.item("Duration", AttributeValue::N(a.to_string()));
        }

        builder = builder.transact_items(TransactWriteItem::builder()
            .put(put.build())
            .build());
    }

    for set in exercise.delete_sets.iter() {
        let delete = Delete::builder()
            .table_name(common::TABLE_USER_SET)
            .key("UserId", AttributeValue::S(user_id.clone()))
            .key("Id", AttributeValue::S(format!("{}#{}#{}", workout_id, exercise_id, set)));

        builder = builder.transact_items(TransactWriteItem::builder()
            .delete(delete.build())
            .build());
    }

    builder.send().await?;

    common::empty_response(StatusCode::OK)
}
