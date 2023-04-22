use std::collections::HashMap;
use aws_sdk_dynamodb::{
    Client,
    error::SdkError,
    operation::update_item::UpdateItemError,
    types::{AttributeValue, Select, WriteRequest, DeleteRequest, ReturnValue, PutRequest},
};
use lambda_http::{Request, http::StatusCode};
use tokio_stream::StreamExt;
use crate::common;

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();

    get_snapshot(db, user_id).await
}

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();

    let user = match common::parse_request_json::<common::User>(&req) {
        Ok(b) => b,
        Err(e) => return e,
    };

    put_snapshot(db, user_id, user).await
}

async fn get_snapshot(db: &Client, user_id: String) -> common::Result {
    // We're not using a read lock. Instead, we check the version before and
    // after the operation. If the version changed, then we have an inconsistent
    // snapshot and we'll have to try again.

    let get_version = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let version = get_version.item()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let collection = common::collection_from_version(version);

    let items = db.query()
        .table_name(common::TABLE_USER)
        .key_condition_expression("UserId = :userId AND begins_with(Id, :collection)")
        .filter_expression("attribute_not_exists(Deleted)")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .expression_attribute_values(
            ":collection",
            AttributeValue::S(common::get_collection_prefix(collection)),
        )
        .select(Select::AllAttributes)
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    let get_version = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let new_version = get_version.item()
        .map_or(0, |i| common::as_number(&i["Version"]));

    if new_version != version {
        common::retry_later_response(0)
    } else {
        common::json_response(
            StatusCode::OK,
            common::db_to_user(version, false, &items),
        )
    }
}

async fn put_snapshot(
    db: &Client,
    user_id: String,
    import_user: common::User<'_>,
) -> common::Result {
    // Acquire the lock. Writes aren't allowed while this lock is valid. Reads
    // are still allowed though. Reads will be on the current collection which
    // won't change while the lock is valid. If this step fails, nothing
    // happens.

    const LOCK_DURATION_S: u64 = 60;

    let now = common::now();
    let now_attr = AttributeValue::N(now.to_string());
    let lock_expire = now + LOCK_DURATION_S;
    let lock_expire_attr = AttributeValue::N(lock_expire.to_string());

    // Unfortunately, there doesn't seem to be a way to do a conditional update
    // and get the item if the condition is true or false. With an UpdateItem,
    // you can get the item if it's true but not false. With TransactWriteItems,
    // it's the reverse. In either case, you can't have both. It's a very
    // frustrating limitation.
    //
    // If we do an UpdateItem, then we'll need to do a GetItem if it fails. In
    // that case, LockedUntil might have changed since we failed to acquire the
    // lock. That doesn't really matter though. A TransactWriteItems would be
    // slower. We're optimizing the common case.

    let acquire_lock_result = db.update_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .update_expression("SET LockedUntil = :lockExpire")
        .condition_expression(
            "attribute_not_exists(LockedUntil) OR LockedUntil <= :now"
        )
        .expression_attribute_values(":lockExpire", lock_expire_attr)
        .expression_attribute_values(":now", now_attr)
        .return_values(ReturnValue::AllOld)
        .send()
        .await;

    let acquire_lock = match acquire_lock_result {
        Ok(o) => o,
        Err(e) => {
            if let SdkError::ServiceError(service_error) = &e {
                if let UpdateItemError::ConditionalCheckFailedException(_) = &service_error.err() {
                    // If the conditional expression is false, then LockedUntil
                    // is in the future but the item could have changed since
                    // then so we can't assume anything about it.

                    let get_lock = db.get_item()
                        .table_name(common::TABLE_USER)
                        .key("UserId", AttributeValue::S(user_id.clone()))
                        .key("Id", AttributeValue::S("VERSION".into()))
                        .send()
                        .await?;

                    let delay = get_lock.item()
                        .and_then(|i| i.get("LockedUntil"))
                        .map(common::as_number::<u64>)
                        .map_or(0, |until| until.saturating_sub(common::now()));

                    return common::retry_later_response(delay);
                }
            }

            return Err(e.into());
        }
    };

    // Get the current collection. We need this to apply the import changes
    // relative to the current state of the database. If this step fails, the
    // database will be read-only until the lock expires. Apart from that, there
    // are no side effects.

    let curr_version = acquire_lock.attributes()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let curr_collection = common::collection_from_version(curr_version);
    let curr_collection_prefix = common::get_collection_prefix(curr_collection);
    let new_collection = curr_collection + 1;
    let new_version = common::version_from_collection(new_collection);

    let curr_items = db.query()
        .table_name(common::TABLE_USER)
        .key_condition_expression("UserId = :userId AND begins_with(Id, :collection)")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .expression_attribute_values(
            ":collection",
            AttributeValue::S(curr_collection_prefix.clone()),
        )
        .select(Select::AllAttributes)
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    let curr_user = common::db_to_user(curr_version, false, &curr_items);

    // Combine the imported collection with the current collection to determine
    // the new collection and then write it out in batches.

    common::batch_write(
        db,
        common::TABLE_USER,
        make_import_batch(user_id.clone(), new_version, &curr_user, &import_user),
    ).await?;

    // Release the lock and switch to the new collection. If this step fails,
    // the database will be read-only until the lock expires. The new collection
    // will remain until it is overwritten by the next import attempt.

    db.update_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .update_expression("REMOVE LockedUntil SET Version = :version")
        .expression_attribute_values(":version", AttributeValue::N(new_version.to_string()))
        .send()
        .await?;

    // Clear out the old collection. If this step fails, then the old collection
    // will remain. There is currently no mechanism to remove the data if this
    // step fails. It's not doing any harm really. It's just sitting there.
    // Given a user ID and the current collection ID, it's easy to query for all
    // of this junk data so perhaps there could be a garbage collection
    // mechanism?

    common::batch_write(
        db,
        common::TABLE_USER,
        make_delete_batch(user_id, &curr_collection_prefix, &curr_user),
    ).await?;

    common::empty_response(StatusCode::OK)
}

fn make_import_batch<'a>(
    user_id: String,
    new_version: u64,
    curr: &common::User<'a>,
    import: &common::User<'a>,
) -> Vec<WriteRequest> {
    let mut requests = Vec::new();

    let new_collection_prefix = common::get_collection_prefix(
        common::collection_from_version(new_version)
    );

    apply_changes::<common::MeasurementSet>(
        &mut requests,
        user_id.clone(),
        &new_collection_prefix,
        new_version,
        import,
        curr,
    );

    apply_changes::<common::Workout>(
        &mut requests,
        user_id.clone(),
        &new_collection_prefix,
        new_version,
        import,
        curr,
    );

    apply_changes::<common::Exercise>(
        &mut requests,
        user_id,
        &new_collection_prefix,
        new_version,
        import,
        curr,
    );

    requests
}

fn apply_changes<'a, T: common::ToDynamoDb<'a> + common::Equivalent + common::UserField<'a>>(
    requests: &mut Vec<WriteRequest>,
    user_id: String,
    collection_prefix: &str,
    version: u64,
    import: &common::User<'a>,
    curr: &common::User<'a>,
) {
    let mut curr_entities = T::extract_from_user(curr).iter()
        .map(|e| (e.get_id(), e))
        .collect::<HashMap<_, _>>();
    let mut curr_deleted_entities: HashMap<&str, u64> = HashMap::from_iter(
        T::extract_deleted_from_user(curr).iter().map(|d| (d.id, d.modified_version))
    );

    for import_entity in T::extract_from_user(import).iter() {
        curr_deleted_entities.remove(import_entity.get_id());

        let mut modified_version = Some(version);

        let entity = if let Some(curr_entity) = curr_entities.remove(import_entity.get_id()) {
            if curr_entity.equiv(import_entity) {
                modified_version = None;
                curr_entity
            } else {
                import_entity
            }
        } else {
            import_entity
        };

        let mut item = HashMap::new();

        item.insert("UserId".into(), AttributeValue::S(user_id.clone()));
        item.insert("Id".into(), AttributeValue::S(
            common::make_key_from_entity(collection_prefix, entity)
        ));
        entity.insert_dynamo_db(&mut item, modified_version);

        requests.push(make_put_request(item));
    }

    for entity in curr_entities.values() {
        let mut item = HashMap::new();

        item.insert("UserId".into(), AttributeValue::S(user_id.clone()));
        item.insert("Id".into(), AttributeValue::S(
            common::make_key_from_entity(collection_prefix, *entity)
        ));
        entity.insert_dynamo_db(&mut item, None);

        requests.push(make_put_request(item));
    }

    for (id, modified_version) in curr_deleted_entities.iter() {
        let mut item = HashMap::new();

        item.insert("UserId".into(), AttributeValue::S(user_id.clone()));
        item.insert("Id".into(), AttributeValue::S(
            common::make_key_from_id::<T>(collection_prefix, id)
        ));
        item.insert("Deleted".into(), AttributeValue::Bool(true));
        item.insert("ModifiedVersion".into(), AttributeValue::N(
            modified_version.to_string()
        ));

        requests.push(make_put_request(item));
    }
}

fn make_put_request(item: HashMap<String, AttributeValue>) -> WriteRequest {
    WriteRequest::builder()
        .put_request(PutRequest::builder()
            .set_item(Some(item))
            .build()
        )
        .build()
}

fn make_delete_batch(
    user_id: String,
    collection_prefix: &str,
    user: &common::User,
) -> Vec<WriteRequest> {
    let mut requests = Vec::new();

    apply_delete::<common::MeasurementSet>(
        &mut requests,
        user_id.clone(),
        collection_prefix,
        user,
    );

    apply_delete::<common::Workout>(
        &mut requests,
        user_id.clone(),
        collection_prefix,
        user,
    );

    apply_delete::<common::Exercise>(
        &mut requests,
        user_id.clone(),
        collection_prefix,
        user,
    );

    requests
}

fn apply_delete<'a, T: common::ToDynamoDb<'a> + common::UserField<'a>>(
    requests: &mut Vec<WriteRequest>,
    user_id: String,
    collection_prefix: &str,
    user: &common::User<'a>,
) {
    for entity in T::extract_from_user(user) {
        requests.push(make_delete_request(
            user_id.clone(),
            common::make_key_from_entity(collection_prefix, entity),
        ));
    }

    for deleted in T::extract_deleted_from_user(user) {
        requests.push(make_delete_request(
            user_id.clone(),
            common::make_key_from_id::<T>(collection_prefix, deleted.id),
        ));
    }
}

fn make_delete_request(user_id: String, key: String) -> WriteRequest {
    WriteRequest::builder()
        .delete_request(
            DeleteRequest::builder()
                .key("UserId", AttributeValue::S(user_id))
                .key("Id", AttributeValue::S(key))
                .build()
        )
        .build()
}
