use std::{collections::HashMap, borrow::Cow};
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
        if version == 0 {
            get_all(db, user_id).await?
        } else {
            get_changed(db, user_id, version).await?
        }
    } else {
        get_all(db, user_id).await?
    };

    Ok(common::with_cors(Response::builder())
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body.into())
        .map_err(Box::new)?)
}

const UUID_LEN: usize = 36;
const DATE_LEN: usize = 10;
const VERSION_LEN: usize = "VERSION".len();
const MEASUREMENT_PREFIX_LEN: usize = "MEASUREMENT#".len();
const MEASUREMENT_SET_LEN: usize = MEASUREMENT_PREFIX_LEN + DATE_LEN;
const WORKOUT_PREFIX_LEN: usize = "WORKOUT#".len();
const WORKOUT_LEN: usize = WORKOUT_PREFIX_LEN + UUID_LEN;
const EXERCISE_LEN: usize = WORKOUT_PREFIX_LEN + 2 * UUID_LEN + 1;

async fn get_all(db: &Client, user_id: String) -> Result<String, Error> {
    let items = db.query()
        .table_name(common::TABLE_USER)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id))
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    Ok(serde_json::to_string(&to_user(0, &items)).unwrap())
}

async fn get_changed(db: &Client, user_id: String, client_version: u32) -> Result<String, Error> {
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

    let version = if let Some(item) = get_version.item() {
        common::as_number(&item["Version"])
    } else {
        0
    };

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

        return Ok(serde_json::to_string(&common::User {
            version,
            measurement_sets: Vec::new(),
            workouts: Vec::new(),
            exercises: Vec::new(),
            deleted_measurement_sets: Vec::new(),
            deleted_workouts: Vec::new(),
            deleted_exercises: Vec::new(),
        }).unwrap());
    }

    // Query for items that were modified after the given version. There's an
    // LSI on the ModifiedVersion but the index only includes the keys as to
    // avoid slowing down writes too much.

    let items = db.query()
        .table_name(common::TABLE_USER)
        .index_name(common::INDEX_MODIFIED_VERSION)
        .key_condition_expression("UserId = :userId AND ModifiedVersion > :clientVersion")
        .expression_attribute_values(":userId", AttributeValue::S(user_id))
        .expression_attribute_values(":clientVersion", AttributeValue::N(client_version.to_string()))
        .select(Select::AllAttributes)
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    Ok(serde_json::to_string(&to_user(version, &items)).unwrap())
}

fn to_user(mut version: u32, items: &Vec<HashMap<String, AttributeValue>>) -> common::User {
    let mut measurement_sets = Vec::new();
    let mut workouts = Vec::new();
    let mut exercises = Vec::new();
    let mut deleted_measurement_sets = Vec::new();
    let mut deleted_workouts = Vec::new();
    let mut deleted_exercises = Vec::new();

    for item in items.iter() {
        let sk = item["Id"].as_s().unwrap();
        let deleted = item.contains_key("Deleted");

        match sk.len() {
            VERSION_LEN => {
                version = common::as_number(&item["Version"]);
            }

            MEASUREMENT_SET_LEN => {
                let measurement_id = &sk[MEASUREMENT_PREFIX_LEN..];
                if deleted {
                    deleted_measurement_sets.push(measurement_id);
                } else {
                    measurement_sets.push(to_measurement_set(measurement_id, item));
                }
            }

            WORKOUT_LEN => {
                let workout_id = &sk[WORKOUT_PREFIX_LEN..];
                if deleted {
                    deleted_workouts.push(workout_id);
                } else {
                    workouts.push(to_workout(workout_id, item));
                }
            }

            EXERCISE_LEN => {
                let workout_exercise_id = &sk[WORKOUT_PREFIX_LEN..];
                if deleted {
                    deleted_exercises.push(workout_exercise_id);
                } else {
                    exercises.push(to_exercise(workout_exercise_id, item));
                }
            }

            _ => unreachable!()
        }
    }

    common::User {
        version,
        measurement_sets,
        workouts,
        exercises,
        deleted_measurement_sets,
        deleted_workouts,
        deleted_exercises,
    }
}

fn to_measurement_set<'a>(
    date: &'a str,
    item: &'a HashMap<String, AttributeValue>,
) -> common::MeasurementSet<'a> {
    common::MeasurementSet {
        date,
        notes: common::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
        measurements: item["Measurements"].as_m().unwrap().iter()
            .map(|(k, v)| (k.as_str(), common::as_number(v)))
            .collect()
    }
}

fn to_workout<'a>(
    workout_id: &'a str,
    item: &'a HashMap<String, AttributeValue>,
) -> common::Workout<'a> {
    common::Workout {
        workout_id,
        start_time: item.get("StartTime").map(|a| a.as_s().unwrap().as_str()),
        finish_time: item.get("FinishTime").map(|a| a.as_s().unwrap().as_str()),
        notes: common::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
    }
}

fn to_exercise<'a>(
    workout_exercise_id: &'a str,
    item: &'a HashMap<String, AttributeValue>,
) -> common::Exercise<'a> {
    common::Exercise {
        workout_exercise_id,
        order: common::as_number(&item["Order"]),
        r#type: common::MaxLenStr(Cow::Borrowed(item["Type"].as_s().unwrap())),
        notes: common::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
        sets: common::MaxLenVec(to_sets(item["Sets"].as_l().unwrap())),
    }
}

fn to_sets(sets: &Vec<AttributeValue>) -> Vec<common::Set> {
    sets.iter()
        .map(|set| {
            let map = set.as_m().unwrap();
            common::Set {
                set_id: common::Uuid(map["SetId"].as_s().unwrap().as_str()),
                repetitions: map.get("Repetitions").map(common::as_number),
                resistance: map.get("Resistance").map(common::as_number),
                speed: map.get("Speed").map(common::as_number),
                distance: map.get("Distance").map(common::as_number),
                duration: map.get("Duration").map(common::as_number),
            }
        })
        .collect()
}
