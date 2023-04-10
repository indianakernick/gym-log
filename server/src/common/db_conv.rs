use std::{collections::HashMap, borrow::Cow};
use aws_sdk_dynamodb::types::AttributeValue;

pub const TABLE_USER: &str = "gym-log.User";
pub const INDEX_MODIFIED_VERSION: &str = "LSI-ModifiedVersion";

pub const COLLECTION_LEN: usize = u32::ilog10(u32::MAX) as usize;

pub fn get_collection_prefix(collection: u32) -> String {
    format!("{collection:0COLLECTION_LEN$}#")
}

pub fn collection_from_version(version: u64) -> u32 {
    (version >> 32) as u32
}

const UUID_LEN: usize = 36;
const DATE_LEN: usize = 10;
const MEASUREMENT_PREFIX_LEN: usize = super::COLLECTION_LEN + "#MEASUREMENT#".len();
const MEASUREMENT_SET_LEN: usize = MEASUREMENT_PREFIX_LEN + DATE_LEN;
const WORKOUT_PREFIX_LEN: usize = super::COLLECTION_LEN + "#WORKOUT#".len();
const WORKOUT_LEN: usize = WORKOUT_PREFIX_LEN + UUID_LEN;
const EXERCISE_LEN: usize = WORKOUT_PREFIX_LEN + 2 * UUID_LEN + 1;

pub fn db_to_user(
    version: u64,
    include_deleted: bool,
    filter_collection: bool,
    items: &Vec<HashMap<String, AttributeValue>>,
) -> super::User {
    let mut measurement_sets = Vec::new();
    let mut workouts = Vec::new();
    let mut exercises = Vec::new();
    let mut deleted_measurement_sets = Vec::new();
    let mut deleted_workouts = Vec::new();
    let mut deleted_exercises = Vec::new();

    let collection = super::get_collection_prefix(
        super::collection_from_version(version)
    );

    for item in items.iter() {
        let sk = item["Id"].as_s().unwrap();
        let deleted = include_deleted && item.contains_key("Deleted");

        // If there is an import in-progress, then there could be two
        // collections so we'll need to filter them. When fetching items for a
        // snapshot, the primary index is used so we don't need to filter it
        // here.

        if filter_collection && !sk.starts_with(&collection) {
            continue;
        }

        match sk.len() {
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

    super::User {
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
) -> super::MeasurementSet<'a> {
    super::MeasurementSet {
        date,
        notes: super::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
        measurements: item["Measurements"].as_m().unwrap().iter()
            .map(|(k, v)| (k.as_str(), super::as_number(v)))
            .collect()
    }
}

fn to_workout<'a>(
    workout_id: &'a str,
    item: &'a HashMap<String, AttributeValue>,
) -> super::Workout<'a> {
    super::Workout {
        workout_id,
        start_time: item.get("StartTime").map(|a| a.as_s().unwrap().as_str()),
        finish_time: item.get("FinishTime").map(|a| a.as_s().unwrap().as_str()),
        notes: super::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
    }
}

fn to_exercise<'a>(
    workout_exercise_id: &'a str,
    item: &'a HashMap<String, AttributeValue>,
) -> super::Exercise<'a> {
    super::Exercise {
        workout_exercise_id,
        order: super::as_number(&item["Order"]),
        r#type: super::MaxLenStr(Cow::Borrowed(item["Type"].as_s().unwrap())),
        notes: super::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
        sets: super::MaxLenVec(to_sets(item["Sets"].as_l().unwrap())),
    }
}

fn to_sets(sets: &Vec<AttributeValue>) -> Vec<super::Set> {
    sets.iter()
        .map(|set| {
            let map = set.as_m().unwrap();
            super::Set {
                set_id: super::Uuid(map["SetId"].as_s().unwrap().as_str()),
                repetitions: map.get("Repetitions").map(super::as_number),
                resistance: map.get("Resistance").map(super::as_number),
                speed: map.get("Speed").map(super::as_number),
                distance: map.get("Distance").map(super::as_number),
                duration: map.get("Duration").map(super::as_number),
            }
        })
        .collect()
}
