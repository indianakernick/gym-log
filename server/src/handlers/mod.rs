pub mod user;
pub mod user_measurement;
pub mod user_workout;
pub mod user_workout_exercise;
pub mod user_workout_order;

pub fn options() -> crate::common::Result {
    crate::common::empty_response(lambda_http::http::StatusCode::OK)
}
