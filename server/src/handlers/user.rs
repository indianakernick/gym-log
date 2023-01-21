use std::collections::HashMap;
use aws_sdk_dynamodb::{model::{AttributeValue, Select}, Client};
use lambda_http::{Request, Response, http::StatusCode, Error, RequestExt};
use tokio_stream::StreamExt;
use crate::common;

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();
    let query_map = req.query_string_parameters();
    let since_version = query_map.first("since");

    let body = if let Some(version) = since_version {
        let version = match version.parse() {
            Ok(v) => v,
            Err(_) => return common::empty_response(StatusCode::BAD_REQUEST),
        };
        get_changed(db, user_id, version).await?
    } else {
        get_all(db, user_id).await?
    };

    Ok(common::with_cors(Response::builder())
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body.into())
        .map_err(Box::new)?)
}

async fn get_all(db: &Client, user_id: String) -> Result<String, Error> {
    let items = db.query()
        .table_name(common::TABLE_USER)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .filter_expression("attribute_not_exists(Deleted)")
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    // M < V < W

    let result = items.binary_search_by(|item| {
        item["Id"].as_s().unwrap().as_str().cmp("VERSION")
    });

    let version;
    let end_measurement;
    let first_workout;

    match result {
        Ok(index) => {
            version = common::as_number(&items[index]["Version"]);
            end_measurement = index;
            first_workout = index + 1;
        }
        Err(index) => {
            version = 0;
            end_measurement = index;
            first_workout = index;
        }
    }

    let measurements = get_all_measurements(items[..end_measurement].iter());
    let workouts = get_all_workouts(items[first_workout..].iter());

    let user = common::User {
        version,
        measurements,
        workouts,
        deleted_measurements: Vec::new(),
        deleted_workouts: Vec::new(),
    };

    Ok(serde_json::to_string(&user).unwrap())
}

async fn get_changed(db: &Client, user_id: String, client_version: u32) -> Result<String, Error> {
    // Get the version first. The objects that we return may have a greater
    // modified version than this if they are modified while we're querying
    // them but that's OK. The client knows that it has at least this version
    // possibly some pieces from a later version. The client could request
    // changes since this version to pick up the things that were modified at a
    // bad time. The version and the modified version are updated in a
    // transaction so the version will never be greater than it should be.

    let version = get_version(db, user_id.clone()).await?;

    // If the client is requesting changes after the current version, then we
    // know that there won't be anything so we can skip the extra queries and
    // return an empty response. If there is only one client making
    // modifications to a particular user's data, then this early-exit is the
    // path that will be taken.

    if version <= client_version {
        return Ok(serde_json::to_string(&common::User {
            version,
            measurements: Vec::new(),
            workouts: Vec::new(),
            deleted_measurements: Vec::new(),
            deleted_workouts: Vec::new(),
        }).unwrap());
    }

    // Query for items that were modified after the given version. There's an
    // LSI on the ModifiedVersion but the index only includes the keys as to
    // avoid slowing down writes too much. This is still better than querying
    // everything and then filtering.

    let query_changed = db.query()
        .table_name(common::TABLE_USER)
        .index_name(common::INDEX_MODIFIED_VERSION)
        .key_condition_expression("UserId = :userId AND Version > :clientVersion")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .expression_attribute_values(":clientVersion", AttributeValue::N(client_version.to_string()))
        .select(Select::AllAttributes)
        .send()
        .await?;

    let items = query_changed.items().unwrap();
    let mut changed_measurements = Vec::new();
    let mut deleted_measurements = Vec::new();
    let mut changed_workout_ids = Vec::new();
    let mut deleted_workouts = Vec::new();

    for changed in items.iter() {
        let id = changed["Id"].as_s().unwrap();
        let is_measurement = id.starts_with("MEASUREMENT#");

        if changed.contains_key("Deleted") {
            if is_measurement {
                deleted_measurements.push(&id["MEASUREMENT#".len()..]);
            } else {
                deleted_workouts.push(&id["WORKOUT#".len()..]);
            }
        } else {
            if is_measurement {
                changed_measurements.push(changed);
            } else {
                changed_workout_ids.push(id.as_str());
            }
        }
    }

    let measurements = get_all_measurements(changed_measurements.iter().copied());

    // For workouts, we need to separately query for the nested exercises and
    // sets. We could attach a modified version to these sub-items to get them
    // in the above query. We might also want to consider storing sets as a list
    // of maps within the exercise item.

    // This isn't very efficient but it works. We're getting there!

    let mut queries = Vec::with_capacity(changed_workout_ids.len());
    let mut workouts = Vec::with_capacity(changed_workout_ids.len());

    for workout_id in changed_workout_ids.iter() {
        queries.push(db.query()
            .table_name(common::TABLE_USER)
            .key_condition_expression("UserId = :userId AND begins_with(Id, :prefix)")
            .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
            .expression_attribute_values(":prefix", AttributeValue::S((*workout_id).into()))
            .send()
            .await?);
    }

    for query in queries.iter() {
        workouts.extend(
            get_all_workouts(query.items().unwrap().iter()).drain(..)
        );
    }

    let user = common::User {
        version,
        measurements,
        workouts,
        deleted_measurements,
        deleted_workouts,
    };

    Ok(serde_json::to_string(&user).unwrap())
}

async fn get_version(db: &Client, user_id: String) -> Result<u32, Error> {
    let get = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    if let Some(item) = get.item() {
        Ok(common::as_number(&item["Version"]))
    } else {
        Ok(0)
    }
}

fn get_all_measurements<'a, I>(items: I) -> Vec<common::Measurement<'a>>
    where I: Iterator<Item=&'a HashMap<String, AttributeValue>>
{
    let mut measurements = Vec::<common::Measurement>::new();

    for item in items {
        measurements.push(common::Measurement {
            measurement_id: item["Id"].as_s().unwrap(),
            modified_version: common::as_number(&item["ModifiedVersion"]),
            r#type: common::MaxLenStr(item["Type"].as_s().unwrap()),
            capture_date: item["CaptureDate"].as_s().unwrap(),
            value: common::as_number(&item["Value"]),
            notes: common::MaxLenStr(item["Notes"].as_s().unwrap()),
        });
    }

    measurements
}

fn get_all_workouts<'a, I>(items: I) -> Vec<common::Workout<'a>>
    where I: Iterator<Item=&'a HashMap<String, AttributeValue>>
{
    let mut workouts = Vec::<common::Workout>::new();
    let mut exercises = Vec::<common::Exercise>::new();
    let mut sets = Vec::<common::Set>::new();

    for item in items {
        const PREFIX_LEN: usize = "WORKOUT#".len();
        const UUID_LEN: usize = 36;
        const WORKOUT_LEN: usize = PREFIX_LEN + UUID_LEN;
        const EXERCISE_LEN: usize = PREFIX_LEN + 2 * UUID_LEN + 1;
        const SET_LEN: usize = PREFIX_LEN + 3 * UUID_LEN + 2;

        let sk = item["Id"].as_s().unwrap();

        match sk.len() {
            WORKOUT_LEN => {
                if !exercises.is_empty() {
                    let last = exercises.len() - 1;
                    sets.sort_unstable_by_key(|s| s.order);
                    exercises[last].sets = std::mem::take(&mut sets);
                }

                if !workouts.is_empty() {
                    let last = workouts.len() - 1;
                    exercises.sort_unstable_by_key(|e| e.order);
                    workouts[last].exercises = std::mem::take(&mut exercises);
                }

                workouts.push(common::Workout {
                    workout_id: &sk[PREFIX_LEN..],
                    modified_version: common::as_number(&item["ModifiedVersion"]),
                    start_time: item.get("StartTime").map(|a| a.as_s().unwrap().as_str()),
                    finish_time: item.get("FinishTime").map(|a| a.as_s().unwrap().as_str()),
                    notes: common::MaxLenStr(item["Notes"].as_s().unwrap()),
                    exercises: Vec::new(),
                });
            }

            EXERCISE_LEN => {
                if !exercises.is_empty() {
                    let last = exercises.len() - 1;
                    exercises.sort_unstable_by_key(|e| e.order);
                    exercises[last].sets = std::mem::take(&mut sets);
                }

                exercises.push(common::Exercise {
                    exercise_id: &sk[WORKOUT_LEN + 1..],
                    order: common::as_number(&item["Order"]),
                    r#type: item["Type"].as_s().unwrap(),
                    notes: item["Notes"].as_s().unwrap(),
                    sets: Vec::new(),
                    delete_sets: Vec::new(),
                });
            }

            SET_LEN => {
                sets.push(common::Set {
                    set_id: &sk[EXERCISE_LEN + 1..],
                    order: common::as_number(&item["Order"]),
                    repetitions: item.get("Repetitions").map(common::as_number),
                    resistance: item.get("Resistance").map(common::as_number),
                    speed: item.get("Speed").map(common::as_number),
                    distance: item.get("Distance").map(common::as_number),
                    duration: item.get("Duration").map(common::as_number),
                });
            }

            _ => unreachable!(),
        }
    }

    if !exercises.is_empty() {
        let last = exercises.len() - 1;
        sets.sort_unstable_by_key(|s| s.order);
        exercises[last].sets = std::mem::take(&mut sets);
    }

    if !workouts.is_empty() {
        let last = workouts.len() - 1;
        exercises.sort_unstable_by_key(|e| e.order);
        workouts[last].exercises = std::mem::take(&mut exercises);
    }

    workouts.sort_by_key(|w| w.start_time);

    workouts
}
