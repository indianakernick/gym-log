use std::{collections::HashMap, borrow::Cow};
use serde::{Serialize, Deserialize};

pub const TABLE_USER: &str = "gym-log.User";
pub const INDEX_MODIFIED_VERSION: &str = "LSI-ModifiedVersion";
pub const MAX_EXERCISES: usize = 25;
pub const MAX_SETS: usize = 25;
pub const MAX_TYPE_LEN: usize = 100;
pub const MAX_NOTES_LEN: usize = 10000;

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    /// The current version of the user's data.
    #[serde(skip_deserializing)]
    pub version: u64,
    #[serde(borrow)]
    pub measurement_sets: Vec<MeasurementSet<'a>>,
    #[serde(borrow)]
    pub workouts: Vec<Workout<'a>>,
    #[serde(borrow)]
    pub exercises: Vec<Exercise<'a>>,
    /// A list of measurements that were deleted since the given version.
    #[serde(borrow)]
    #[serde(skip_deserializing)]
    pub deleted_measurement_sets: Vec<&'a str>,
    /// A list of workouts that were deleted since the given version.
    #[serde(borrow)]
    #[serde(skip_deserializing)]
    pub deleted_workouts: Vec<&'a str>,
    /// A list of exercises that were deleted since the given version.
    #[serde(borrow)]
    #[serde(skip_deserializing)]
    pub deleted_exercises: Vec<&'a str>,
}

// I'm really struggling to find a good word for this.
#[derive(Serialize, Deserialize)]
pub struct MeasurementSet<'a> {
    /// The date that the measurements were captured in ISO 8601 precise to the
    /// day.
    #[serde(skip_deserializing)]
    pub date: &'a str,
    /// Any user provided notes associated with the measurements on this day.
    #[serde(borrow)]
    pub notes: MaxLenStr<'a, MAX_NOTES_LEN>,
    /// The measurements captured on this day as a map from type to value.
    pub measurements: HashMap<&'a str, f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Workout<'a> {
    /// UUID of the workout.
    #[serde(skip_deserializing)]
    pub workout_id: &'a str,
    /// The time that the workout started in ISO 8601 precise to the second.
    #[serde(deserialize_with = "deserialize_time")]
    pub start_time: Option<&'a str>,
    /// The time that the workout finished in ISO 8601 precise to the second.
    #[serde(deserialize_with = "deserialize_time")]
    pub finish_time: Option<&'a str>,
    /// Any user provided notes associated with the workout.
    #[serde(borrow)]
    pub notes: MaxLenStr<'a, MAX_NOTES_LEN>,
}

#[derive(Serialize, Deserialize)]
pub struct Exercise<'a> {
    /// UUID of the workout concatenated with the UUID of the exercise separated
    /// by a `#`.
    #[serde(skip_deserializing)]
    pub workout_exercise_id: &'a str,
    /// Index of the exercise within the workout.
    pub order: u32,
    /// The type of exercise which defines the meaning of various properties on
    /// sets.
    #[serde(borrow)]
    pub r#type: MaxLenStr<'a, MAX_TYPE_LEN>,
    /// Any user provided notes associated with the exercise.
    #[serde(borrow)]
    pub notes: MaxLenStr<'a, MAX_NOTES_LEN>,
    /// The sets within the exercise.
    #[serde(borrow)]
    pub sets: MaxLenVec<Set<'a>, MAX_SETS>,
}

#[derive(Serialize, Deserialize)]
pub struct Set<'a> {
    /// UUID of the set.
    #[serde(borrow)]
    pub set_id: Uuid<'a>,
    /// The number of repetitions for an exercise type that requires it.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetitions: Option<u32>,
    /// The resistance level which may be unit-less, kilograms or degrees for an
    /// exercise type that requires it.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resistance: Option<u32>,
    /// The speed in kilometres per hour for an exercise type that requires it.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<u32>,
    /// The distance in meters for an exercise type that requires it.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<u32>,
    /// The duration in seconds for an exercise type that requires it.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
}

fn deserialize_time<'de: 'a, 'a, D>(d: D) -> Result<Option<&'a str>, D::Error>
    where D: serde::Deserializer<'de>
{
    let os = Option::<&str>::deserialize(d)?;
    if let Some(s) = os {
        match chrono::NaiveDate::parse_from_str(s, "%FT%TZ") {
            Ok(_) => Ok(os),
            Err(e) => Err(serde::de::Error::custom(e))
        }
    } else {
        Ok(None)
    }
}

/// A wrapper around a &str that validates it is a UUID when deserializing.
#[repr(transparent)]
#[derive(Serialize)]
pub struct Uuid<'a>(pub &'a str);

impl<'de: 'a, 'a> Deserialize<'de> for Uuid<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = <&str>::deserialize(deserializer)?;
        if super::is_uuid(s) {
            Ok(Uuid(s))
        } else {
            Err(serde::de::Error::custom("invalid UUID"))
        }
    }
}

/// A wrapper around a Vec<T> that validates its length when deserializing.
#[repr(transparent)]
#[derive(Serialize)]
pub struct MaxLenVec<T, const MAX_LEN: usize>(pub Vec<T>);

impl<'de, T: Deserialize<'de>, const MAX_LEN: usize> Deserialize<'de> for MaxLenVec<T, MAX_LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let v = Vec::<T>::deserialize(deserializer)?;
        if v.len() <= MAX_LEN {
            Ok(MaxLenVec(v))
        } else {
            let msg = format!("no more than {} items", MAX_LEN);
            Err(serde::de::Error::invalid_length(v.len(), &msg.as_str()))
        }
    }
}

/// A wrapper around a Cow<str> that validates its length when deserializing.
#[repr(transparent)]
#[derive(Serialize)]
pub struct MaxLenStr<'a, const MAX_LEN: usize>(pub Cow<'a, str>);

impl<'de: 'a, 'a, const MAX_LEN: usize> Deserialize<'de> for MaxLenStr<'a, MAX_LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = <Cow<str>>::deserialize(deserializer)?;
        if s.len() <= MAX_LEN {
            Ok(MaxLenStr(s))
        } else {
            let msg = format!("no more than {} characters", MAX_LEN);
            Err(serde::de::Error::invalid_length(s.len(), &msg.as_str()))
        }
    }
}
