use std::ops::ControlFlow;
use aws_sdk_dynamodb::{
    error::{TransactWriteItemsError, TransactWriteItemsErrorKind},
    model::{TransactWriteItem, AttributeValue, Update, ReturnValuesOnConditionCheckFailure, put, Put, CancellationReason, ConditionCheck},
    types::SdkError, client::fluent_builders::TransactWriteItems,
};
use lambda_http::{Request, http::StatusCode};
use serde::Deserialize;

// When an item is edited in some way, the modified version of that item is
// updated and the root version for all of the user's data is also updated. The
// root version is used to detect client cache invalidation. The item modified
// version is used to find the items required to update the client cache.
//
// Both the item version and root version are updated together in a transaction.
// The client includes its version in the request body. If the client's version
// is not equal to the current version, then the client's cache is out of date.
// The transaction will be canceled and a 409 response will be returned. The
// client will need to get the changes made since its own version before trying
// again. It may find that it's trying to modify something that was modified by
// another client of the same user (a merge conflict), in which case the user
// will need to be prompted on how to resolve it.
//
// When deleting specifically, there are other failure cases. If the request
// references an item that doesn't exist at all, that's an indication of a bug
// somewhere and a 404 is returned. If the referenced item exists but has
// already been deleted then it could be because the client's cache is out of
// date. If cache validation has already been checked then this is also a bug
// and a 404 is returned.
//
// If the operation is successful, the client can update its cache and
// increment its version. What the client had before is what the database had
// before. So after the client applies the modification, and the database
// applies the modification, we know that they're synchronised. The client will
// not need to download this change when it requests changes.

#[derive(Deserialize)]
pub struct VersionDeleteReq {
    pub version: u32,
}

#[derive(Deserialize)]
pub struct VersionModifyReq<T> {
    pub version: u32,
    // Should we flatten here?
    pub item: T,
}

pub async fn version_delete(
    req: &Request,
    item_id: String,
) -> super::Result {
    let client_version = match super::parse_request_json::<VersionDeleteReq>(req) {
        Ok(b) => b.version,
        Err(r) => return r,
    };

    version_apply(
        req,
        client_version,
        |builder, user_id, new_version| {
            version_delete_item(builder, user_id, item_id, new_version)
        },
        |reasons| {
            if reasons[0].code() == Some("ConditionalCheckFailed") {
                return ControlFlow::Break(super::empty_response(StatusCode::NOT_FOUND));
            }

            ControlFlow::Continue(())
        }
    ).await
}

pub async fn version_modify<'r, T, P>(
    req: &'r Request,
    patch: P,
) -> super::Result
    where
        T: Deserialize<'r>,
        P: FnOnce(TransactWriteItems, &T, String, String) -> TransactWriteItems,
{
    version_modify_checked(req, patch, |_| ControlFlow::Continue(())).await
}

pub async fn version_modify_checked<'r, T, P, C>(
    req: &'r Request,
    patch: P,
    check: C,
) -> super::Result
    where
        T: Deserialize<'r>,
        P: FnOnce(TransactWriteItems, &T, String, String) -> TransactWriteItems,
        C: FnOnce(&[CancellationReason]) -> ControlFlow<super::Result, ()>,
{
    let body = match super::parse_request_json::<VersionModifyReq<T>>(req) {
        Ok(b) => b,
        Err(e) => return e,
    };

    version_apply(
        req,
        body.version,
        |builder, user_id, new_version| {
            patch(builder, &body.item, user_id, new_version)
        },
        check,
    ).await
}

pub fn version_put_item<T, P>(
    item_id: String,
    patch: P,
) -> impl FnOnce(TransactWriteItems, &T, String, String) -> TransactWriteItems
    where P: FnOnce(put::Builder, &T) -> put::Builder
{
    move |builder, item, user_id, new_version| {
        let put = Put::builder()
            .table_name(super::TABLE_USER)
            .item("UserId", AttributeValue::S(user_id))
            .item("Id", AttributeValue::S(item_id.clone()))
            .item("ModifiedVersion", AttributeValue::N(new_version));
        builder.transact_items(
            TransactWriteItem::builder()
                .put(patch(put, item).build())
                .build()
        )
    }
}

pub fn version_delete_item(
    builder: TransactWriteItems,
    user_id: String,
    item_id: String,
    new_version: String,
) -> TransactWriteItems {
    builder.transact_items(TransactWriteItem::builder()
        .update(Update::builder()
            .table_name(super::TABLE_USER)
            .key("UserId", AttributeValue::S(user_id))
            .key("Id", AttributeValue::S(item_id))
            .expression_attribute_values(":newVersion", AttributeValue::N(new_version))
            .expression_attribute_values(":deleted", AttributeValue::Bool(true))
            .condition_expression("attribute_exists(UserId) AND attribute_not_exists(Deleted)")
            .update_expression("SET ModifiedVersion = :newVersion, Deleted = :deleted")
            .build())
        .build())
}

pub fn check_exists(
    builder: TransactWriteItems,
    user_id: String,
    item_id: String,
) -> TransactWriteItems {
    builder.transact_items(TransactWriteItem::builder()
        .condition_check(ConditionCheck::builder()
            .table_name(super::TABLE_USER)
            .key("UserId", AttributeValue::S(user_id))
            .key("Id", AttributeValue::S(item_id))
            .condition_expression("attribute_exists(UserId) AND attribute_not_exists(Deleted)")
            .build())
        .build())
}

pub async fn version_apply<P, C>(
    req: &Request,
    client_version: u32,
    patch: P,
    check: C,
) -> super::Result
    where
        P: FnOnce(TransactWriteItems, String, String) -> TransactWriteItems,
        C: FnOnce(&[CancellationReason]) -> ControlFlow<super::Result, ()>,
{
    let db = super::get_db_client();
    let user_id = super::get_user_id(req);
    let new_version = (client_version + 1).to_string();
    let client_version = client_version.to_string();

    let builder = db.transact_write_items()
        .transact_items(TransactWriteItem::builder()
            .update(Update::builder()
                .table_name(super::TABLE_USER)
                .key("UserId", AttributeValue::S(user_id.clone()))
                .key("Id", AttributeValue::S("VERSION".into()))
                .expression_attribute_values(":clientVersion", AttributeValue::N(client_version.clone()))
                .expression_attribute_values(":newVersion", AttributeValue::N(new_version.clone()))
                .condition_expression("attribute_not_exists(Version) OR Version = :clientVersion")
                .update_expression("SET Version = :newVersion")
                .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::AllOld)
                .build())
            .build());
    let result = patch(builder, user_id, new_version).send().await;

    match result {
        Ok(_) => super::empty_response(StatusCode::OK),
        Err(e) => {
            if let Some(reasons) = cancellation_reasons(&e) {
                if reasons[0].code() == Some("ConditionalCheckFailed") {
                    if let Some(item) = reasons[0].item() {
                        let old_version = item["Version"].as_n().unwrap();
                        if old_version != &client_version {
                            return super::empty_response(StatusCode::CONFLICT);
                        }
                    }
                }

                if let ControlFlow::Break(r) = check(&reasons[1..]) {
                    return r;
                }
            }

            Err(e.into())
        }
    }
}

fn cancellation_reasons(error: &SdkError<TransactWriteItemsError>) -> Option<&[CancellationReason]> {
    if let SdkError::ServiceError(service_error) = error {
        if let TransactWriteItemsErrorKind::TransactionCanceledException(cancel) = &service_error.err().kind {
            return cancel.cancellation_reasons();
        }
    }
    None
}
