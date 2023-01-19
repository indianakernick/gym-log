use aws_sdk_dynamodb::{
    model::{AttributeValue, TransactWriteItem, Update, ReturnValuesOnConditionCheckFailure, Put},
    types::SdkError,
    error::TransactWriteItemsErrorKind
};
use lambda_http::{Request, RequestExt, http::StatusCode};
use serde::{Deserialize, Serialize};
use super::{common, model};

#[derive(Serialize, Deserialize)]
struct MaxModTime {
    max_modified_time: u128,
}

pub async fn delete(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();
    let client_time = match common::parse_request_json::<MaxModTime>(&req) {
        Ok(b) => b.max_modified_time.to_string(),
        Err(r) => return r,
    };

    if !common::is_uuid(measurement_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    let db = common::get_db_client();
    let mut current_time = common::get_timestamp();

    loop {
        let current_time_str = current_time.to_string();

        let result = db.transact_write_items()
            .transact_items(TransactWriteItem::builder()
                .update(Update::builder()
                    .table_name(common::TABLE_USER)
                    .key("UserId", AttributeValue::S(user_id.clone()))
                    .key("Id", AttributeValue::S("VERSION".into()))
                    .expression_attribute_values(":newTime", AttributeValue::N(current_time_str.clone()))
                    .expression_attribute_values(":clientTime", AttributeValue::N(client_time.clone()))
                    .condition_expression(":newTime > MaxModifiedTime AND :clientTime = MaxModifiedTime")
                    .update_expression("SET MaxModifiedTime = :newTime")
                    .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::AllOld)
                    .build())
                .build())
            .transact_items(TransactWriteItem::builder()
                .update(Update::builder()
                    .table_name(common::TABLE_USER)
                    .key("UserId", AttributeValue::S(user_id.clone()))
                    .key("Id", AttributeValue::S(format!("MEASUREMENT#{}", measurement_id)))
                    .expression_attribute_values(":newTime", AttributeValue::N(current_time_str))
                    .expression_attribute_values(":deleted", AttributeValue::Bool(true))
                    .condition_expression("attribute_exists(UserId) AND attribute_not_exists(Deleted)")
                    .update_expression("SET ModifiedTime = :newTime, Deleted = :deleted")
                    .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::AllOld)
                    .build())
                .build())
            .send()
            .await;

        match result {
            Ok(_) => return common::json_response(StatusCode::OK, MaxModTime {
                max_modified_time: current_time,
            }),
            Err(e) => {
                // There are four failure cases that we want to explicitly
                // handle. Any other kind of failure results in a 500.
                //
                //  1. The target doesn't exist and never existed.
                //     - Return a 404.
                //  2. The target did exist but has since been deleted.
                //     - Return a 404.
                //  3. The client's cache has been invalidated.
                //     - Return a 409.
                //  4. This Lambda instance's clock is slightly behind the clock
                //     of the instance that just recently modified some of of
                //     the user's data.
                //     - Try again with a later time.

                if let SdkError::ServiceError(ref service_err) = e {
                    if let TransactWriteItemsErrorKind::TransactionCanceledException(except) = &service_err.err().kind {
                        if let Some(reasons) = except.cancellation_reasons() {
                            if reasons[1].code() == Some("ConditionalCheckFailed") {
                                if let Some(item) = reasons[1].item() {
                                    if item.contains_key("Deleted") {
                                        // Case 2
                                        return common::empty_response(StatusCode::NOT_FOUND);
                                    }
                                } else {
                                    // Case 1
                                    return common::empty_response(StatusCode::NOT_FOUND);
                                }
                            }

                            if reasons[0].code() == Some("ConditionalCheckFailed") {
                                if let Some(item) = reasons[0].item() {
                                    // The MaxModifiedTime must exist if the
                                    // item we're deleting exists.
                                    let old_time = item["MaxModifiedTime"].as_n().unwrap();
                                    if old_time != &client_time {
                                        // Case 3
                                        return common::empty_response(StatusCode::CONFLICT);
                                    }

                                    // Case 4
                                    current_time = old_time.parse::<u128>().unwrap() + 1;
                                    continue;
                                }
                            }
                        }
                    }
                }

                return Err(e.into());
            }
        }
    }
}

#[derive(Deserialize)]
struct ModifyReq<T> {
    max_modified_time: u128,
    // Should we flatten here?
    item: T,
}

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if let Err(e) = common::validate_uuid(measurement_id) {
        return e;
    }

    let req = match common::parse_request_json::<ModifyReq<model::Measurement>>(&req) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let client_time = req.max_modified_time.to_string();

    // Could use a custom deserialization function so that serde generates the
    // error message.
    if let Err(e) = chrono::NaiveDate::parse_from_str(req.item.capture_date, "%F") {
        return common::error_response(StatusCode::BAD_REQUEST, &format!("Invalid capture_date: {}", e));
    }

    let db = common::get_db_client();
    let mut current_time = common::get_timestamp();

    loop {
        let current_time_str = current_time.to_string();

        let result = db.transact_write_items()
            .transact_items(TransactWriteItem::builder()
                .update(Update::builder()
                    .table_name(common::TABLE_USER)
                    .key("UserId", AttributeValue::S(user_id.clone()))
                    .key("Id", AttributeValue::S("VERSION".into()))
                    .expression_attribute_values(":newTime", AttributeValue::N(current_time_str.clone()))
                    .expression_attribute_values(":clientTime", AttributeValue::N(client_time.clone()))
                    .condition_expression("attribute_not_exists(MaxModifiedTime) OR (:newTime > MaxModifiedTime AND :clientTime = MaxModifiedTime)")
                    .update_expression("SET MaxModifiedTime = :newTime")
                    .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::AllOld)
                    .build())
                .build())
            .transact_items(TransactWriteItem::builder()
                .put(Put::builder()
                    .table_name(common::TABLE_USER)
                    .item("UserId", AttributeValue::S(user_id.clone()))
                    .item("Id", AttributeValue::S(format!("MEASUREMENT#{}", measurement_id)))
                    .item("ModifiedTime", AttributeValue::N(current_time_str))
                    .item("Type", AttributeValue::S(req.item.r#type.into()))
                    .item("CaptureDate", AttributeValue::S(req.item.capture_date.into()))
                    .item("Value", AttributeValue::N(req.item.value.to_string()))
                    .item("Notes", AttributeValue::S(req.item.notes.into()))
                    .build())
                .build())
            .send()
            .await;

        match result {
            Ok(_) => return common::json_response(StatusCode::OK, MaxModTime {
                max_modified_time: current_time,
            }),
            Err(e) => {
                // There are four failure cases that we want to explicitly
                // handle. Any other kind of failure results in a 500.
                //
                //  1. The client's cache has been invalidated.
                //     - Return a 409.
                //  2. This Lambda instance's clock is slightly behind the clock
                //     of the instance that just recently modified some of of
                //     the user's data.
                //     - Try again with a later time.

                if let SdkError::ServiceError(ref service_err) = e {
                    if let TransactWriteItemsErrorKind::TransactionCanceledException(except) = &service_err.err().kind {
                        if let Some(reasons) = except.cancellation_reasons() {
                            if reasons[0].code() == Some("ConditionalCheckFailed") {
                                if let Some(item) = reasons[0].item() {
                                    // If the MaxModifiedTime didn't exist, then
                                    // the condition would have passed.
                                    let old_time = item["MaxModifiedTime"].as_n().unwrap();
                                    if old_time != &client_time {
                                        // Case 1
                                        return common::empty_response(StatusCode::CONFLICT);
                                    }

                                    // Case 2
                                    current_time = old_time.parse::<u128>().unwrap() + 1;
                                    continue;
                                }
                            }
                        }
                    }
                }

                return Err(e.into());
            }
        }
    }
}
