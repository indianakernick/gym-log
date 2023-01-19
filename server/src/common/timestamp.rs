use aws_sdk_dynamodb::{
    error::{TransactWriteItemsError, TransactWriteItemsErrorKind},
    model::{TransactWriteItem, AttributeValue, Update, ReturnValuesOnConditionCheckFailure, put, Put, CancellationReason},
    types::SdkError,
};
use lambda_http::{Request, http::StatusCode};
use serde::{Serialize, Deserialize};

// When an item is edited in some way, the modified date of that item is
// updated and the max modified date for all of the user's data is also set. The
// max modified date is used to detect client cache invalidation. The item
// modified date is used to find the items required to update the client cache.
//
// Both the item and max modified date are updated together in a transaction.
// This transaction is conditional and there are a couple of possible failure
// cases that are handled. The client includes its current max modified date in
// the request body and the new max modified date is included in a success
// response.
//
// The first failure case is that the client's max modified date is not equal to
// the current max modified date. This means that the client's cache is out of
// date. A 409 response will be returned and the client will need to get the
// changes made since it's own max modified date. It may find that it's trying
// to modify something that was modified by another client of the same user (a
// merge conflict) in which case the user will need to be prompted on how to
// resolve it. Once that's done, the client can try again.
//
// The second failure case is that the new max modified date is less than or
// equal to the current max modified date. This is a very rare edge case but it
// can happen if a user is using multiple devices at once (or perhaps connecting
// multiple offline devices to the internet at almost the same time). It's
// possible for multiple instances of a Lambda function to be running in
// parallel. These instances may be on separate hardware which means that their
// clocks will not be perfectly synced. This is unlike an RDS where there is one
// central clock. So if this happens, the whole operation is attempted again
// using the current max modified date plus 1 instead of the current time.
//
// When deleting specifically, there are other failure cases. If the request
// references an item that doesn't exist at all, that's an indication of a bug
// somewhere and a 404 is returned. If the referenced item exists but has
// already been deleted then it could be because the client's cache is out of
// date. If cache validation has already been checked then this is also a bug
// and a 404 is returned.
//
// If the operation is successful, the new max modified date is returned to the
// client. The client can then update it's cache. What the client had before is
// what the database had before. So after the client applies the modification,
// and the database applies the modification, we know that they're synchronised.
// The client will not need to download this change when it requests changes.

pub async fn timestamp_delete(
    req: &Request,
    item_id: String,
) -> super::Result {
    let db = super::get_db_client();
    let user_id = super::get_user_id(req);
    let client_time = match super::parse_request_json::<TimestampDeleteReq>(&req) {
        Ok(b) => b.max_modified_time.to_string(),
        Err(r) => return r,
    };
    let mut current_time = get_timestamp();

    loop {
        let current_time_str = current_time.to_string();

        let result = db.transact_write_items()
            .transact_items(update_max_modified_time(
                user_id.clone(), current_time_str.clone(), client_time.clone()
            ))
            .transact_items(update_delete(
                user_id.clone(), item_id.clone(), current_time_str
            ))
            .send()
            .await;

        match result {
            Ok(_) => return super::json_response(StatusCode::OK, TimestampRes {
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
                //  3. The target doesn't exist and never existed.
                //     - Return a 404.
                //  4. The target did exist but has since been deleted.
                //     - Return a 404.

                if let Some(reasons) = cancellation_reasons(&e) {
                    if reasons[0].code() == Some("ConditionalCheckFailed") {
                        if let Some(item) = reasons[0].item() {
                            let old_time = item["MaxModifiedTime"].as_n().unwrap();
                            if old_time != &client_time {
                                // Case 1
                                return super::empty_response(StatusCode::CONFLICT);
                            }

                            // Case 2
                            current_time = old_time.parse::<u128>().unwrap() + 1;
                            continue;
                        }
                    }

                    if reasons[1].code() == Some("ConditionalCheckFailed") {
                        if let Some(item) = reasons[1].item() {
                            if item.contains_key("Deleted") {
                                // Case 4
                                return super::empty_response(StatusCode::NOT_FOUND);
                            }
                        } else {
                            // Case 3
                            return super::empty_response(StatusCode::NOT_FOUND);
                        }
                    }
                }

                return Err(e.into());
            }
        }
    }
}

pub async fn timestamp_modify<'r, T, F>(
    req: &'r Request,
    item_id: String,
    patch: F,
) -> super::Result
    where
        T: Deserialize<'r>,
        F: Fn(put::Builder, &T) -> put::Builder,
{
    let db = super::get_db_client();
    let user_id = super::get_user_id(req);
    let req = match super::parse_request_json::<TimestampModifyReq<T>>(req) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let client_time = req.max_modified_time.to_string();
    let mut current_time = get_timestamp();

    loop {
        let current_time_str = current_time.to_string();

        let result = db.transact_write_items()
            .transact_items(update_max_modified_time(
                user_id.clone(), current_time_str.clone(), client_time.clone()
            ))
            .transact_items(put_item(
                user_id.clone(),
                item_id.clone(),
                current_time_str,
                &req.item,
                &patch,
            ))
            .send()
            .await;

            match result {
                Ok(_) => return super::json_response(StatusCode::OK, TimestampRes {
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

                    if let Some(reasons) = cancellation_reasons(&e) {
                        if reasons[0].code() == Some("ConditionalCheckFailed") {
                            if let Some(item) = reasons[0].item() {
                                let old_time = item["MaxModifiedTime"].as_n().unwrap();
                                if old_time != &client_time {
                                    // Case 1
                                    return super::empty_response(StatusCode::CONFLICT);
                                }

                                // Case 2
                                current_time = old_time.parse::<u128>().unwrap() + 1;
                                continue;
                            }
                        }
                    }

                    return Err(e.into());
                }
            }
    }
}

#[derive(Serialize, Deserialize)]
struct TimestampDeleteReq {
    max_modified_time: u128,
}

type TimestampRes = TimestampDeleteReq;

#[derive(Deserialize)]
struct TimestampModifyReq<T> {
    max_modified_time: u128,
    // Should we flatten here?
    item: T,
}

fn get_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn update_max_modified_time(
    user_id: String,
    current_time: String,
    client_time: String,
) -> TransactWriteItem {
    TransactWriteItem::builder()
        .update(Update::builder()
            .table_name(super::TABLE_USER)
            .key("UserId", AttributeValue::S(user_id))
            .key("Id", AttributeValue::S("VERSION".into()))
            .expression_attribute_values(":newTime", AttributeValue::N(current_time))
            .expression_attribute_values(":clientTime", AttributeValue::N(client_time))
            .condition_expression("attribute_not_exists(MaxModifiedTime) OR (:newTime > MaxModifiedTime AND :clientTime = MaxModifiedTime)")
            .update_expression("SET MaxModifiedTime = :newTime")
            .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::AllOld)
            .build())
        .build()
}

fn update_delete(
    user_id: String,
    item_id: String,
    current_time: String,
) -> TransactWriteItem {
    TransactWriteItem::builder()
        .update(Update::builder()
            .table_name(super::TABLE_USER)
            .key("UserId", AttributeValue::S(user_id))
            .key("Id", AttributeValue::S(item_id))
            .expression_attribute_values(":newTime", AttributeValue::N(current_time))
            .expression_attribute_values(":deleted", AttributeValue::Bool(true))
            .condition_expression("attribute_exists(UserId) AND attribute_not_exists(Deleted)")
            .update_expression("SET ModifiedTime = :newTime, Deleted = :deleted")
            .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::AllOld)
            .build())
        .build()
}

fn put_item<T, F>(
    user_id: String,
    item_id: String,
    current_time: String,
    item: &T,
    patch: F
) -> TransactWriteItem
    where F: Fn(put::Builder, &T) -> put::Builder
{
    let put = Put::builder()
        .table_name(super::TABLE_USER)
        .item("UserId", AttributeValue::S(user_id))
        .item("Id", AttributeValue::S(item_id))
        .item("ModifiedTime", AttributeValue::N(current_time));
    TransactWriteItem::builder()
        .put(patch(put, item).build())
        .build()
}

fn cancellation_reasons(error: &SdkError<TransactWriteItemsError>) -> Option<&[CancellationReason]> {
    if let SdkError::ServiceError(service_error) = error {
        if let TransactWriteItemsErrorKind::TransactionCanceledException(cancel) = &service_error.err().kind {
            return cancel.cancellation_reasons();
        }
    }
    None
}
