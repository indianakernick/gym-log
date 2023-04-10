use aws_sdk_dynamodb::{types::{AttributeValue, Select, ReturnValue, DeleteRequest, WriteRequest}, Client};
use lambda_http::{Request, http::StatusCode, RequestExt, Error};
use tokio_stream::StreamExt;
use crate::common;

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();
    let query_map = req.query_string_parameters();
    let since_version = query_map.first("since");

    let result = if let Some(version) = since_version {
        let version = match version.parse() {
            Ok(v) => v,
            Err(_) => return common::empty_response(StatusCode::BAD_REQUEST),
        };
        get_changed(db, user_id, version).await
    } else {
        get_snapshot(db, user_id).await
    };

    result
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

async fn get_changed(db: &Client, user_id: String, client_version: u64) -> common::Result {
    // Get the version first. The objects that we return may have a greater
    // modified version than this if they are modified while we're querying
    // them but that's OK. The client knows that it has at least this version
    // and possibly some pieces from a later version. The client could request
    // changes since this version to pick up the things that were modified at a
    // bad time. The version and the modified version are updated in a
    // transaction so the version will never be greater than it should be.

    let get_version = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let version = get_version.item()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let collection = common::collection_from_version(version);

    // If the client is requesting changes after the current version, then we
    // know that there won't be anything so we can skip the extra queries and
    // return an empty response. If there is only one client making
    // modifications to a particular user's data, then this early-exit is the
    // path that will be taken.

    if version <= client_version {
        // TODO: since this is so common, perhaps make it smaller
        // we could use the If-None-Match header where the ETag is the version
        // number. Reaching this branch would end up being a 304 response with
        // the version in the ETag header.

        return common::json_response(StatusCode::OK, common::User {
            version,
            measurement_sets: Vec::new(),
            workouts: Vec::new(),
            exercises: Vec::new(),
            deleted_measurement_sets: Vec::new(),
            deleted_workouts: Vec::new(),
            deleted_exercises: Vec::new(),
        });
    }

    // Query for items that were modified after the given version. There's an
    // LSI on the ModifiedVersion but the index only includes the keys as to
    // avoid slowing down writes too much.

    let items = db.query()
        .table_name(common::TABLE_USER)
        .index_name(common::INDEX_MODIFIED_VERSION)
        .key_condition_expression("UserId = :userId AND ModifiedVersion > :clientVersion")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .expression_attribute_values(
            ":clientVersion",
            AttributeValue::N(client_version.to_string()),
        )
        .select(Select::AllAttributes)
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    // Now that we've queried for all of the items, the version is checked
    // again.

    let get_version = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let new_version = get_version.item()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let new_collection = common::collection_from_version(new_version);

    // If the collection changed, then an import completed while we were
    // reading. So what we read was probably in the process of being deleted
    // while we were reading it. This means what we've read is probably
    // incomplete and shouldn't be trusted. We should tell the client to try
    // again.

    // If the version changed, then we have a mix of the state of the previous
    // version and the state of the new version. This is fine when fetching the
    // changes but not for exporting a full snapshot.

    if new_collection != collection {
        common::retry_later_response(0)
    } else {
        common::json_response(StatusCode::OK, common::db_to_user(version, false, &items))
    }
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
        common::json_response(StatusCode::OK, common::db_to_user(version, true, &items))
    }
}

async fn put_snapshot(db: &Client, user_id: String, import_user: common::User<'_>) -> common::Result {
    // Acquire the lock. Writes aren't allowed while this lock is valid. Reads
    // are still allowed though. Reads will be on the current collection which
    // won't change while the lock is valid. If this step fails, nothing
    // happens.

    const LOCK_DURATION: u64 = 60;

    let now = common::now();
    let now_attr = AttributeValue::N(now.to_string());
    let lock_expire = now + LOCK_DURATION;
    let lock_expire_attr = AttributeValue::N(lock_expire.to_string());

    let acquire_lock = db.update_item()
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
        .await?;

    // Get the current collection. We need this to apply the import changes
    // relative to the current state of the database. If this step fails, the
    // database will be read-only until the lock expires. Apart from that, there
    // are no side effects.

    let curr_version = acquire_lock.attributes()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let curr_collection = common::collection_from_version(curr_version);
    let curr_collection_prefix = common::get_collection_prefix(curr_collection);
    let new_collection = curr_collection + 1;
    let new_version = ((new_collection as u64) << 32) | (curr_version & 0xFFFFFFFF);

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

    // TODO: compare curr_user and import_user to do the actual import.

    // Release the lock and switch to the new collection. If this step fails,
    // the database will be read-only until the lock expires. The new collection
    // will remain until it is overwritten by the next import attempt.

    db.update_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .update_expression("REMOVE LockedUntil, SET Version = :version")
        .expression_attribute_values(":version", AttributeValue::N(new_version.to_string()))
        .send()
        .await?;

    // Clear out the old collection. If this step fails, then the old collection
    // will remain. There is currently no mechanism to remove the data if this
    // step fails. It's not doing any harm really. It's just sitting there.

    execute_batch(
        db,
        make_delete_batch(user_id, &curr_collection_prefix, &curr_user),
    ).await?;

    common::empty_response(StatusCode::OK)
}

async fn execute_batch(db: &Client, requests: Vec<WriteRequest>) -> Result<(), Error> {
    const MAX_BATCH_SIZE: usize = 25;

    let mut unprocessed = Vec::new();
    let mut processed = 0;

    while !unprocessed.is_empty() || processed < requests.len() {
        let mut batch;
        let remaining = requests.len() - processed;

        if !unprocessed.is_empty() {
            batch = unprocessed;
            let processing = remaining.min(MAX_BATCH_SIZE - batch.len());
            batch.extend_from_slice(&requests[processed..processed + processing]);
            processed += processing;
        } else {
            let processing = remaining.min(MAX_BATCH_SIZE);
            batch = Vec::from(&requests[processed..processed + processing]);
            processed += processing;
        }

        let batch_write = db.batch_write_item()
            .request_items(common::TABLE_USER, batch)
            .send()
            .await?;

        unprocessed = batch_write.unprocessed_items()
            .and_then(|map| map.get(common::TABLE_USER))
            .cloned()
            .unwrap_or_default();
    }

    Ok(())
}

fn make_delete_batch(
    user_id: String,
    collection_prefix: &str,
    user: &common::User,
) -> Vec<WriteRequest> {
    let mut requests = Vec::new();

    for m in user.measurement_sets.iter() {
        requests.push(make_delete_request(
            user_id.clone(),
            format!("{collection_prefix}MEASUREMENT#{}", m.date),
        ));
    }
    for m in user.deleted_measurement_sets.iter() {
        requests.push(make_delete_request(
            user_id.clone(),
            format!("{collection_prefix}MEASUREMENT#{m}"),
        ));
    }

    for w in user.workouts.iter() {
        requests.push(make_delete_request(
            user_id.clone(),
            format!("{collection_prefix}WORKOUT#{}", w.workout_id),
        ));
    }
    for w in user.deleted_workouts.iter() {
        requests.push(make_delete_request(
            user_id.clone(),
            format!("{collection_prefix}WORKOUT#{w}"),
        ));
    }

    for e in user.exercises.iter() {
        requests.push(make_delete_request(
            user_id.clone(),
            format!("{collection_prefix}WORKOUT#{}", e.workout_exercise_id),
        ));
    }
    for e in user.deleted_exercises.iter() {
        requests.push(make_delete_request(
            user_id.clone(),
            format!("{collection_prefix}WORKOUT#{e}"),
        ));
    }

    requests
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
