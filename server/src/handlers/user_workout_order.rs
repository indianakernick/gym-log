use std::ops::ControlFlow;
use aws_sdk_dynamodb::model::{TransactWriteItem, Update, AttributeValue};
use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

type Exercises<'a> = common::MaxLenVec<common::Uuid<'a>, { common::MAX_EXERCISES }>;

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    if let Err(e) = common::validate_uuid(workout_id) {
        return e;
    }

    common::version_modify_checked(
        &req,
        |mut builder, exercises: &Exercises, user_id, new_version| {
            builder = builder.transact_items(TransactWriteItem::builder()
                .update(Update::builder()
                    .table_name(common::TABLE_USER)
                    .key("UserId", AttributeValue::S(user_id.clone()))
                    .key("Id", AttributeValue::S(format!("WORKOUT#{workout_id}")))
                    .update_expression("SET ModifiedVersion = :newVersion")
                    .expression_attribute_values(":newVersion", AttributeValue::N(new_version))
                    .condition_expression("attribute_exists(UserId) AND attribute_not_exists(Deleted)")
                    .build())
                .build());

            for (i, exercise) in exercises.0.iter().map(|e| e.0).enumerate() {
                builder = builder.transact_items(TransactWriteItem::builder()
                    .update(Update::builder()
                        .table_name(common::TABLE_USER)
                        .key("UserId", AttributeValue::S(user_id.clone()))
                        .key("Id", AttributeValue::S(format!("WORKOUT#{workout_id}#{exercise}")))
                        .update_expression("SET #order = :order")
                        .expression_attribute_names("#order", "Order")
                        .expression_attribute_values(":order", AttributeValue::N(i.to_string()))
                        .condition_expression("attribute_exists(UserId)")
                        .build())
                    .build());
            }

            builder
        },
        |reasons| {
            if reasons[0].code() == Some("ConditionalCheckFailed") {
                return ControlFlow::Break(common::empty_response(StatusCode::NOT_FOUND));
            }

            for (i, reason) in reasons[1..].iter().enumerate() {
                if reason.code() == Some("ConditionalCheckFailed") {
                    return ControlFlow::Break(common::error_response(
                        StatusCode::BAD_REQUEST,
                        &format!("exercise referenced by ID {i} doesn't exist"),
                    ));
                }
            }

            ControlFlow::Continue(())
        }
    ).await
}
