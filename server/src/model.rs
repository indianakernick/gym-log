use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    #[serde(borrow)]
    pub measurements: Vec<Measurement<'a>>,
    #[serde(borrow)]
    pub workouts: Vec<Workout<'a>>,
}

#[derive(Serialize, Deserialize)]
pub struct Measurement<'a> {
    /// UUID of the measurement.
    #[serde(skip_deserializing)]
    pub measurement_id: &'a str,
    /// Type of measurement. The client defines the meaning of this.
    pub r#type: &'a str,
    /// The date that the measurement was captured in ISO 8601 precise to the
    /// day.
    pub capture_date: &'a str,
    /// The value of the measurement whose meaning depends on the type.
    pub value: f64,
    /// Any user provided notes associated with the measurement.
    pub notes: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct Workout<'a> {
    /// UUID of the exercise.
    #[serde(skip_deserializing)]
    pub workout_id: &'a str,
    /// The time that the workout started in ISO 8601 precise to the second.
    pub start_time: Option<&'a str>,
    /// The time that the workout finished in ISO 8601 precise to the second.
    pub finish_time: Option<&'a str>,
    /// Any user provided notes associated with the workout.
    pub notes: &'a str,
    /// The exercises within a workout.
    #[serde(borrow)]
    #[serde(skip_deserializing)]
    pub exercises: Vec<Exercise<'a>>,
}

#[derive(Serialize, Deserialize)]
pub struct Exercise<'a> {
    /// UUID of the exercise.
    #[serde(skip_deserializing)]
    pub exercise_id: &'a str,
    /// Index of the exercise within the workout.
    pub order: u32,
    /// The type of exercise which defines the meaning of various properties on
    /// sets.
    pub r#type: &'a str,
    /// Any user provided notes associated with the exercise.
    pub notes: &'a str,
    /// The sets within the exercise.
    #[serde(borrow)]
    pub sets: Vec<Set<'a>>
}

#[derive(Serialize, Deserialize)]
pub struct Set<'a> {
    /// UUID of the set.
    pub set_id: &'a str,
    /// Index of the set within the exercise.
    pub order: u32,
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
