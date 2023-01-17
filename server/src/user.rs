use std::collections::HashMap;
use aws_sdk_dynamodb::{model::{AttributeValue, ReturnValue, TransactGetItem, Get}, Client};
use lambda_http::{Request, Response, http::StatusCode, Error, RequestExt};
use super::{common, model};

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();
    let query_map = req.query_string_parameters();
    // The query string is included in the browser's cache key for the CORS
    // preflight request. So the browser will do a preflight request every time
    // the version changes. Perhaps pass this information via the request body
    // to get around this.
    let since_version = query_map.first("since_version");

    let body = if let Some(version) = since_version {
        let version = match version.parse() {
            Ok(v) => v,
            Err(_) => return common::empty_response(StatusCode::BAD_REQUEST),
        };
        if version > 0 {
            get_changed(db, user_id, version).await?
        } else {
            get_all(db, user_id).await?
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

async fn get_all(db: &Client, user_id: String) -> Result<String, Error> {
    let get_version = db.update_item()
        .table_name(common::TABLE_USER_SET)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .update_expression("SET Version = if_not_exists(Version, :version)")
        .expression_attribute_values(":version", AttributeValue::N("0".into()))
        .return_values(ReturnValue::UpdatedOld)
        .send()
        .await?;

    let version_str = get_version.attributes().unwrap()["Version"].as_n().unwrap();
    let version = version_str.parse().unwrap();

    let measurements_query = db.query()
        .table_name(common::TABLE_USER_MEASUREMENT)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .filter_expression("attribute_not_exists(Deleted)")
        .send()
        .await?;

    let measurements = get_all_measurements(measurements_query.items().unwrap().iter());

    let workout_query = db.query()
        .table_name(common::TABLE_USER_SET)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id))
        .send()
        .await?;

    let workouts = get_all_workouts(workout_query.items().unwrap().iter());

    let user = model::User {
        version,
        measurements,
        workouts,
        deleted_measurements: Vec::new(),
        deleted_workouts: Vec::new(),
    };

    Ok(serde_json::to_string(&user).unwrap())
}

async fn get_changed(db: &Client, user_id: String, version: u32) -> Result<String, Error> {
    let get_version = db.get_item()
        .table_name(common::TABLE_USER_SET)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let version_str = get_version.item().unwrap()["Version"].as_n().unwrap();
    let current_version = version_str.parse().unwrap();

    let query_changed_measurements = db.query()
        .table_name(common::TABLE_USER_MEASUREMENT)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .filter_expression("ModifiedVersion > :version")
        .expression_attribute_values(":version", AttributeValue::N(version.to_string()))
        .projection_expression("MeasurementId, Deleted")
        .send()
        .await?;

    let items = query_changed_measurements.items().unwrap();
    let mut deleted_measurements = Vec::new();
    let mut measurements_result = None;
    let mut builder = db.transact_get_items();
    let mut got_item = false;

    for changed in items.iter() {
        let id = &changed["MeasurementId"];
        let deleted = changed.contains_key("Deleted");

        if deleted {
            deleted_measurements.push(id.as_s().unwrap().as_str());
            continue;
        }

        builder = builder.transact_items(TransactGetItem::builder()
            .get(Get::builder()
                .table_name(common::TABLE_USER_MEASUREMENT)
                .key("UserId", AttributeValue::S(user_id.clone()))
                .key("MeasurementId", id.clone())
                .build())
            .build());

        got_item = true;
    }

    let measurements = if got_item {
        measurements_result = Some(builder.send().await?);
        let responses = measurements_result.as_ref().unwrap().responses().unwrap();
        get_all_measurements(responses.iter()
            .map(|r| r.item().unwrap()))
    } else {
        Vec::new()
    };

    // This is pretty inefficient. There would need to be changes to the sort
    // key to query only the workouts and not everything. Otherwise we're
    // running the filter expression on every workout, exercise and set for a
    // user.
    let query_changed_workouts = db.query()
        .table_name(common::TABLE_USER_SET)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .filter_expression("attribute_exists(ModifiedVersion) AND ModifiedVersion > :version")
        .expression_attribute_values(":version", AttributeValue::N(version.to_string()))
        .projection_expression("Id, Deleted")
        .send()
        .await?;

    let items = query_changed_workouts.items().unwrap();
    let mut deleted_workouts = Vec::new();
    let mut workouts_result = None;
    let mut builder = db.transact_get_items();
    let mut got_item = false;

    for changed in items.iter() {
        let id = &changed["Id"];
        let deleted = changed.contains_key("Deleted");

        if deleted {
            deleted_workouts.push(id.as_s().unwrap().as_str());
            continue;
        }

        builder = builder.transact_items(TransactGetItem::builder()
            .get(Get::builder()
                .table_name(common::TABLE_USER_SET)
                .key("UserId", AttributeValue::S(user_id.clone()))
                .key("Id", id.clone())
                .build())
            .build());

        got_item = true;
    }

    let workouts = if got_item {
        workouts_result = Some(builder.send().await?);
        let responses = workouts_result.as_ref().unwrap().responses().unwrap();
        get_all_workouts(responses.iter()
            .map(|r| r.item().unwrap()))
    } else {
        Vec::new()
    };

    let user = model::User {
        version: current_version,
        measurements,
        workouts,
        deleted_measurements,
        deleted_workouts,
    };

    Ok(serde_json::to_string(&user).unwrap())
}

fn get_all_measurements<'a, I>(items: I) -> Vec<model::Measurement<'a>>
    where I: Iterator<Item=&'a HashMap<String, AttributeValue>>
{
    let mut measurements = Vec::<model::Measurement>::new();

    for item in items {
        measurements.push(model::Measurement {
            measurement_id: item["MeasurementId"].as_s().unwrap(),
            modified_version: item["ModifiedVersion"].as_n().unwrap().parse().unwrap(),
            r#type: item["MeasurementType"].as_s().unwrap(),
            capture_date: item["CaptureDate"].as_s().unwrap(),
            value: item["Value"].as_n().unwrap().parse().unwrap(),
            notes: item["Notes"].as_s().unwrap(),
        });
    }

    measurements
}

fn get_all_workouts<'a, I>(items: I) -> Vec<model::Workout<'a>>
    where I: Iterator<Item=&'a HashMap<String, AttributeValue>>
{
    let mut workouts = Vec::<model::Workout>::new();
    let mut exercises = Vec::<model::Exercise>::new();
    let mut sets = Vec::<model::Set>::new();

    for item in items {
        const UUID_LEN: usize = 36;
        const UUID_LEN_2: usize = 2 * UUID_LEN + 1;
        const UUID_LEN_3: usize = 3 * UUID_LEN + 2;

        let sk = item["Id"].as_s().unwrap();

        // Remove this when the SK is redesigned
        if sk == "VERSION" {
            continue;
        }

        match sk.len() {
            UUID_LEN => {
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

                workouts.push(model::Workout {
                    workout_id: &sk,
                    modified_version: item["ModifiedVersion"].as_n().unwrap().parse().unwrap(),
                    start_time: item.get("StartTime").map(|a| a.as_s().unwrap().as_str()),
                    finish_time: item.get("FinishTime").map(|a| a.as_s().unwrap().as_str()),
                    notes: item["WorkoutNotes"].as_s().unwrap(),
                    exercises: Vec::new(),
                });
            }

            UUID_LEN_2 => {
                if !exercises.is_empty() {
                    let last = exercises.len() - 1;
                    exercises.sort_unstable_by_key(|e| e.order);
                    exercises[last].sets = std::mem::take(&mut sets);
                }

                exercises.push(model::Exercise {
                    exercise_id: &sk[UUID_LEN + 1..],
                    order: item["ExerciseOrder"].as_n().unwrap().parse().unwrap(),
                    r#type: item["ExerciseType"].as_s().unwrap(),
                    notes: item["ExerciseNotes"].as_s().unwrap(),
                    sets: Vec::new(),
                    delete_sets: Vec::new(),
                });
            }

            UUID_LEN_3 => {
                sets.push(model::Set {
                    set_id: &sk[UUID_LEN_2 + 1..],
                    order: item["SetOrder"].as_n().unwrap().parse().unwrap(),
                    repetitions: item.get("Repetitions").map(|a| a.as_n().unwrap().parse().unwrap()),
                    resistance: item.get("Resistance").map(|a| a.as_n().unwrap().parse().unwrap()),
                    speed: item.get("Speed").map(|a| a.as_n().unwrap().parse().unwrap()),
                    distance: item.get("Distance").map(|a| a.as_n().unwrap().parse().unwrap()),
                    duration: item.get("Duration").map(|a| a.as_n().unwrap().parse().unwrap()),
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
