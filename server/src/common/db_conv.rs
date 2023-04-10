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

pub fn version_from_collection(collection: u32) -> u64 {
    (collection as u64) << 32
}

pub fn db_to_user(
    version: u64,
    include_deleted: bool,
    filter_collection: bool,
    items: &Vec<DynamoDbItem>,
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
            super::MeasurementSet::KEY_LEN => {
                let measurement_id = &sk[super::MeasurementSet::FULL_PREFIX_LEN..];
                if deleted {
                    deleted_measurement_sets.push(measurement_id);
                } else {
                    measurement_sets.push(super::MeasurementSet::from_dynamo_db(
                        measurement_id,
                        item,
                    ));
                }
            }

            super::Workout::KEY_LEN => {
                let workout_id = &sk[super::Workout::FULL_PREFIX_LEN..];
                if deleted {
                    deleted_workouts.push(workout_id);
                } else {
                    workouts.push(super::Workout::from_dynamo_db(
                        workout_id,
                        item,
                    ));
                }
            }

            super::Exercise::KEY_LEN => {
                let workout_exercise_id = &sk[super::Exercise::FULL_PREFIX_LEN..];
                if deleted {
                    deleted_exercises.push(workout_exercise_id);
                } else {
                    exercises.push(super::Exercise::from_dynamo_db(
                        workout_exercise_id,
                        item,
                    ));
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

pub trait Identifiable<'a> {
    const ID_LEN: usize;

    /// Get the primary ID of this entity.
    fn get_id(&self) -> &'a str;
}

pub trait Equivalent {
    /// Compare this entity to another entity without considering the primary ID
    /// or the modified version.
    fn equiv(&self, other: &Self) -> bool;
}

type DynamoDbItem = HashMap<String, AttributeValue>;

pub trait ToDynamoDb<'a>: Identifiable<'a> {
    const KEY_PREFIX: &'static str;
    const FULL_PREFIX_LEN: usize = COLLECTION_LEN + 1 + Self::KEY_PREFIX.len();
    const KEY_LEN: usize = Self::FULL_PREFIX_LEN + Self::ID_LEN;

    fn insert_dynamo_db(&self,
        item: &mut DynamoDbItem,
        modified_version: Option<u64>,
    );
}

pub trait FromDynamoDb<'a> {
    fn from_dynamo_db(id: &'a str, item: &'a DynamoDbItem) -> Self;
}

// -------- Identifiable -------- //

const UUID_LEN: usize = 36;
const DATE_LEN: usize = 10;

impl<'a> Identifiable<'a> for super::MeasurementSet<'a> {
    const ID_LEN: usize = DATE_LEN;

    fn get_id(&self) -> &'a str {
        self.date
    }
}

impl<'a> Identifiable<'a> for super::Workout<'a> {
    const ID_LEN: usize = UUID_LEN;

    fn get_id(&self) -> &'a str {
        self.workout_id
    }
}

impl<'a> Identifiable<'a> for super::Exercise<'a> {
    const ID_LEN: usize = 2 * UUID_LEN + 1;

    fn get_id(&self) -> &'a str {
        self.workout_exercise_id
    }
}

// -------- Equivalent -------- //

impl Equivalent for super::MeasurementSet<'_> {
    fn equiv(&self, other: &Self) -> bool {
        self.notes.0 == other.notes.0
            && self.measurements == other.measurements
    }
}

impl Equivalent for super::Workout<'_> {
    fn equiv(&self, other: &Self) -> bool {
        self.start_time == other.start_time
            && self.finish_time == other.finish_time
            && self.notes.0 == other.notes.0
    }
}

impl Equivalent for super::Exercise<'_> {
    fn equiv(&self, other: &Self) -> bool {
        self.order == other.order
            && self.r#type.0 == other.r#type.0
            && self.notes.0 == other.notes.0
            && self.sets.0.len() == other.sets.0.len()
            && self.sets.0.iter()
                .zip(other.sets.0.iter())
                .all(|(a, b)| a.equiv(b))
    }
}

impl Equivalent for super::Set<'_> {
    fn equiv(&self, other: &Self) -> bool {
        self.set_id.0 == other.set_id.0
            && self.repetitions == other.repetitions
            && self.resistance == other.resistance
            && self.speed == other.speed
            && self.distance == other.distance
            && self.duration == other.duration
    }
}

// -------- ToDynamoDb -------- //

impl<'a> ToDynamoDb<'a> for super::MeasurementSet<'a> {
    const KEY_PREFIX: &'static str = "MEASUREMENT#";

    fn insert_dynamo_db(&self,
        item: &mut DynamoDbItem,
        modified_version: Option<u64>,
    ) {
        item.insert("Notes".into(), AttributeValue::S(
            self.notes.0.as_ref().to_owned()
        ));

        item.insert("Measurements".into(), AttributeValue::M(
            self.measurements.iter()
                .map(|(k, v)| (
                    String::from(*k),
                    AttributeValue::N(v.to_string()),
                ))
                .collect()
        ));

        insert_modified_version(item, self.modified_version, modified_version);
    }
}

impl<'a> ToDynamoDb<'a> for super::Workout<'a> {
    const KEY_PREFIX: &'static str = "WORKOUT#";

    fn insert_dynamo_db(&self,
        item: &mut DynamoDbItem,
        modified_version: Option<u64>,
    ) {
        if let Some(dt) = self.start_time {
            item.insert("StartTime".into(), AttributeValue::S(dt.to_owned()));
        }

        if let Some(dt) = self.finish_time {
            item.insert("FinishTime".into(), AttributeValue::S(dt.to_owned()));
        }

        item.insert("Notes".into(), AttributeValue::S(
            self.notes.0.as_ref().to_owned()
        ));

        insert_modified_version(item, self.modified_version, modified_version);
    }
}

impl<'a> ToDynamoDb<'a> for super::Exercise<'a> {
    const KEY_PREFIX: &'static str = "WORKOUT#";

    fn insert_dynamo_db(&self,
        item: &mut DynamoDbItem,
        modified_version: Option<u64>,
    ) {
        item.insert("Order".into(), AttributeValue::N(self.order.to_string()));

        item.insert("Type".into(), AttributeValue::S(
            self.r#type.0.as_ref().to_string()
        ));

        item.insert("Notes".into(), AttributeValue::S(
            self.notes.0.as_ref().to_owned()
        ));

        item.insert("Sets".into(), AttributeValue::L(
            self.sets.0.iter()
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

        insert_modified_version(item, self.modified_version, modified_version);
    }
}

fn insert_modified_version(
    item: &mut DynamoDbItem,
    entity_version: u64,
    other_version: Option<u64>,
) {
    item.insert("ModifiedVersion".into(), AttributeValue::N(
        other_version.unwrap_or(entity_version).to_string()
    ));
}

// -------- FromDynamoDb -------- //

impl<'a> FromDynamoDb<'a> for super::MeasurementSet<'a> {
    fn from_dynamo_db(id: &'a str, item: &'a DynamoDbItem) -> Self {
        Self {
            date: id,
            notes: super::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
            measurements: item["Measurements"].as_m().unwrap().iter()
                .map(|(k, v)| (k.as_str(), super::as_number(v)))
                .collect(),
            modified_version: super::as_number(&item["ModifiedVersion"]),
        }
    }
}

impl<'a> FromDynamoDb<'a> for super::Workout<'a> {
    fn from_dynamo_db(id: &'a str, item: &'a DynamoDbItem) -> Self {
        Self {
            workout_id: id,
            start_time: item.get("StartTime").map(|a| a.as_s().unwrap().as_str()),
            finish_time: item.get("FinishTime").map(|a| a.as_s().unwrap().as_str()),
            notes: super::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
            modified_version: super::as_number(&item["ModifiedVersion"]),
        }
    }
}

impl<'a> FromDynamoDb<'a> for super::Exercise<'a> {
    fn from_dynamo_db(id: &'a str, item: &'a DynamoDbItem) -> Self {
        Self {
            workout_exercise_id: id,
            order: super::as_number(&item["Order"]),
            r#type: super::MaxLenStr(Cow::Borrowed(item["Type"].as_s().unwrap())),
            notes: super::MaxLenStr(Cow::Borrowed(item["Notes"].as_s().unwrap())),
            sets: super::MaxLenVec(sets_from_dynamo_db(item["Sets"].as_l().unwrap())),
            modified_version: super::as_number(&item["ModifiedVersion"]),
        }
    }
}

fn sets_from_dynamo_db(sets: &Vec<AttributeValue>) -> Vec<super::Set> {
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

// -------- Utils -------- //

pub fn make_key_from_id<'a, T: ToDynamoDb<'a>>(collection_prefix: &str, id: &str) -> String {
    format!("{collection_prefix}{}{id}", T::KEY_PREFIX)
}

pub fn make_key_from_entity<'a, T: ToDynamoDb<'a>>(
    collection_prefix: &str,
    entity: &T,
) -> String {
    make_key_from_id::<T>(collection_prefix, entity.get_id())
}
