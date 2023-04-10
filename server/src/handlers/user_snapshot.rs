use std::collections::{HashMap, HashSet};
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
            common::db_to_user(version, false, false, &items),
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

    let curr_user = common::db_to_user(curr_version, true, false, &curr_items);

    // Combine the imported collection with the current collection to determine
    // the new collection and then write it out in batches.

    common::batch_write(
        db,
        common::TABLE_USER,
        make_import_batch(user_id.clone(), new_version, &curr_user, import_user),
    ).await?;

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
    mut import: common::User<'a>,
) -> Vec<WriteRequest> {
    let mut requests = Vec::new();

    let curr_measurement_sets = curr.measurement_sets.iter()
        .map(|m| (m.date, m))
        .collect::<HashMap<_, _>>();
    let curr_deleted_measurement_sets: HashSet<&str> = HashSet::from_iter(
        curr.deleted_measurement_sets.iter().copied()
    );

    let curr_workouts = curr.workouts.iter()
        .map(|w| (w.workout_id, w))
        .collect::<HashMap<_, _>>();
    let curr_deleted_workouts: HashSet<&str> = HashSet::from_iter(
        curr.deleted_workouts.iter().copied()
    );

    let curr_exercises = curr.exercises.iter()
        .map(|e| (e.workout_exercise_id, e))
        .collect::<HashMap<_, _>>();
    let curr_deleted_exercises: HashSet<&str> = HashSet::from_iter(
        curr.deleted_exercises.iter().copied()
    );

    let new_collection_prefix = common::get_collection_prefix(
        common::collection_from_version(new_version)
    );

    apply_changes(
        &mut requests,
        user_id.clone(),
        &new_collection_prefix,
        &mut import.measurement_sets,
        curr_measurement_sets,
        curr_deleted_measurement_sets,
        |a, b| a.notes.0 == b.notes.0 && a.measurements == b.measurements,
        |e| e.date,
        |item, collection_prefix, id| {
            item.insert("Id".into(), AttributeValue::S(
                format!("{collection_prefix}MEASUREMENT#{id}")
            ));
        },
        |item, entity| {
            item.insert("Notes".into(), AttributeValue::S(
                entity.notes.0.as_ref().to_owned()
            ));
            item.insert("Measurements".into(), AttributeValue::M(
                entity.measurements.iter()
                    .map(|(k, v)| (
                        String::from(*k),
                        AttributeValue::N(v.to_string()),
                    ))
                    .collect()
            ));
            item.insert("ModifiedVersion".into(), AttributeValue::N(
                entity.modified_version.to_string()
            ));
        }
    );

    apply_changes(
        &mut requests,
        user_id.clone(),
        &new_collection_prefix,
        &mut import.workouts,
        curr_workouts,
        curr_deleted_workouts,
        |a, b| {
            a.start_time == b.start_time
                && a.finish_time == b.finish_time
                && a.notes.0 == b.notes.0
        },
        |e| e.workout_id,
        |item, collection_prefix, id| {
            item.insert("Id".into(), AttributeValue::S(
                format!("{collection_prefix}WORKOUT#{id}")
            ));
        },
        |item, entity| {
            if let Some(dt) = entity.start_time {
                item.insert("StartTime".into(), AttributeValue::S(dt.to_owned()));
            }
            if let Some(dt) = entity.finish_time {
                item.insert("FinishTime".into(), AttributeValue::S(dt.to_owned()));
            }
            item.insert("Notes".into(), AttributeValue::S(
                entity.notes.0.as_ref().to_owned()
            ));
            item.insert("ModifiedVersion".into(), AttributeValue::N(
                entity.modified_version.to_string()
            ));
        }
    );

    apply_changes(
        &mut requests,
        user_id.clone(),
        &new_collection_prefix,
        &mut import.exercises,
        curr_exercises,
        curr_deleted_exercises,
        |a, b| {
            a.order == b.order
                && a.r#type.0 == b.r#type.0
                && a.notes.0 == b.notes.0
                && a.sets.0.len() == b.sets.0.len()
                && a.sets.0.iter().zip(b.sets.0.iter())
                    .all(|(a, b)| {
                        a.set_id.0 == b.set_id.0
                            && a.repetitions == b.repetitions
                            && a.resistance == b.resistance
                            && a.speed == b.speed
                            && a.distance == b.distance
                            && a.duration == b.duration
                    })
        },
        |m| m.workout_exercise_id,
        |item, collection_prefix, id| {
            item.insert("Id".into(), AttributeValue::S(
                format!("{collection_prefix}WORKOUT#{id}")
            ));
        },
        |item, entity| {
            item.insert("Order".into(), AttributeValue::N(entity.order.to_string()));
            item.insert("Type".into(), AttributeValue::S(entity.r#type.0.as_ref().to_string()));
            item.insert("Notes".into(), AttributeValue::S(
                entity.notes.0.as_ref().to_owned()
            ));
            item.insert("Sets".into(), AttributeValue::L(
                entity.sets.0.iter()
                    .map(|set| {
                        let mut map = HashMap::new();

                        map.insert("SetId".into(), AttributeValue::S(set.set_id.0.into()));

                        if let Some(a) = set.repetitions {
                            map.insert("Repetitions".into(), AttributeValue::N(a.to_string()));
                        }

                        if let Some(a) = set.resistance {
                            map.insert("Resistance".into(), AttributeValue::N(a.to_string()));
                        }

                        if let Some(a) = set.speed {
                            map.insert("Speed".into(), AttributeValue::N(a.to_string()));
                        }

                        if let Some(a) = set.distance {
                            map.insert("Distance".into(), AttributeValue::N(a.to_string()));
                        }

                        if let Some(a) = set.duration {
                            map.insert("Duration".into(), AttributeValue::N(a.to_string()));
                        }

                        AttributeValue::M(map)
                    })
                    .collect()
            ));
            item.insert("ModifiedVersion".into(), AttributeValue::N(
                entity.modified_version.to_string()
            ));
        }
    );

    requests
}

fn apply_changes<T, Equal, GetId, InsertKey, InsertValue>(
    requests: &mut Vec<WriteRequest>,
    user_id: String,
    collection_prefix: &str,
    import_entities: &mut [T],
    mut curr_entities: HashMap<&str, &T>,
    mut curr_deleted_entities: HashSet<&str>,
    equal: Equal,
    get_id: GetId,
    insert_key: InsertKey,
    insert_value: InsertValue,
)
    where
        Equal: Fn(&T, &T) -> bool,
        GetId: Fn(&T) -> &str,
        InsertKey: Fn(&mut HashMap<String, AttributeValue>, &str, &str),
        InsertValue: Fn(&mut HashMap<String, AttributeValue>, &T),
{
    for import_entity in import_entities.iter_mut() {
        curr_deleted_entities.remove(get_id(import_entity));

        let entity = if let Some(curr_entity) = curr_entities.remove(get_id(import_entity)) {
            if equal(curr_entity, import_entity) {
                curr_entity
            } else {
                &*import_entity
            }
        } else {
            import_entity
        };

        let mut item = HashMap::new();

        item.insert("UserId".into(), AttributeValue::S(user_id.clone()));
        insert_key(&mut item, collection_prefix, get_id(entity));
        insert_value(&mut item, entity);

        requests.push(make_put_request(item));
    }

    for entity in curr_entities.values() {
        let mut item = HashMap::new();

        item.insert("UserId".into(), AttributeValue::S(user_id.clone()));
        insert_key(&mut item, collection_prefix, get_id(entity));
        insert_value(&mut item, entity);

        requests.push(make_put_request(item));
    }

    for id in curr_deleted_entities.iter() {
        let mut item = HashMap::new();

        item.insert("UserId".into(), AttributeValue::S(user_id.clone()));
        insert_key(&mut item, collection_prefix, id);
        item.insert("Deleted".into(), AttributeValue::Bool(true));

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
