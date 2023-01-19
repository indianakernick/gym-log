use aws_sdk_dynamodb::model::{TransactWriteItem, Update, AttributeValue};
use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let workout_id = params.first("workoutId").unwrap();

    let exercises = match common::parse_request_json::<Vec<&str>>(&req) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let db = common::get_db_client();
    let mut builder = db.transact_write_items();

    for (i, exercise) in exercises.iter().enumerate() {
        builder = builder.transact_items(TransactWriteItem::builder()
            .update(Update::builder()
                .table_name(common::TABLE_USER_SET)
                .key("UserId", AttributeValue::S(user_id.clone()))
                .key("Id", AttributeValue::S(format!("{}#{}", workout_id, exercise)))
                .update_expression("SET ExerciseOrder = :order")
                .expression_attribute_values(":order", AttributeValue::N(i.to_string()))
                .build())
            .build());
    }

    builder.send().await?;

    common::empty_response(StatusCode::OK)
}
