use aws_sdk_dynamodb::{model::AttributeValue, output::QueryOutput};
use lambda_http::{Request, Response, http::StatusCode};
use super::{common, model};

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();

    let output = db.query()
        .table_name(common::TABLE_USER_MEASUREMENT)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .send()
        .await?;

    let measurements = get_all_measurements(&output);

    let output = db.query()
        .table_name(common::TABLE_USER_SET)
        .key_condition_expression("UserId = :userId")
        .expression_attribute_values(":userId", AttributeValue::S(user_id))
        .send()
        .await?;

    let workouts = get_all_workouts(&output);

    let user = model::User {
        measurements,
        workouts,
    };

    Ok(common::with_cors(Response::builder())
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&user).unwrap().into())
        .map_err(Box::new)?)
}

fn get_all_measurements<'a>(output: &'a QueryOutput) -> Vec<model::Measurement<'a>> {
    let items = output.items().unwrap();
    let mut measurements = Vec::<model::Measurement>::new();

    for item in items {
        measurements.push(model::Measurement {
            measurement_id: item["MeasurementId"].as_s().unwrap(),
            r#type: item["MeasurementType"].as_s().unwrap(),
            capture_date: item["CaptureDate"].as_s().unwrap(),
            value: item["Value"].as_n().unwrap().parse().unwrap(),
            notes: item["Notes"].as_s().unwrap(),
        });
    }

    measurements
}

fn get_all_workouts<'a>(output: &'a QueryOutput) -> Vec<model::Workout<'a>> {
    let items = output.items().unwrap();
    let mut workouts = Vec::<model::Workout>::new();
    let mut exercises = Vec::<model::Exercise>::new();
    let mut sets = Vec::<model::Set>::new();

    for item in items {
        const UUID_LEN: usize = 36;
        const UUID_LEN_2: usize = 2 * UUID_LEN + 1;
        const UUID_LEN_3: usize = 3 * UUID_LEN + 2;

        let sk = item["Id"].as_s().unwrap();

        match sk.len() {
            UUID_LEN => {
                if !exercises.is_empty() {
                    let last = exercises.len() - 1;
                    exercises[last].sets = std::mem::take(&mut sets);
                }

                if !workouts.is_empty() {
                    let last = workouts.len() - 1;
                    workouts[last].exercises = std::mem::take(&mut exercises);
                }

                workouts.push(model::Workout {
                    workout_id: &sk,
                    start_time: item.get("StartTime").map(|a| a.as_s().unwrap().as_str()),
                    finish_time: item.get("FinishTime").map(|a| a.as_s().unwrap().as_str()),
                    notes: item["WorkoutNotes"].as_s().unwrap(),
                    exercises: Vec::new(),
                });
            }

            UUID_LEN_2 => {
                if !exercises.is_empty() {
                    let last = exercises.len() - 1;
                    exercises[last].sets = std::mem::take(&mut sets);
                }

                exercises.push(model::Exercise {
                    exercise_id: &sk[UUID_LEN + 1..],
                    order: item["ExerciseOrder"].as_n().unwrap().parse().unwrap(),
                    r#type: item["ExerciseType"].as_s().unwrap(),
                    notes: item["ExerciseNotes"].as_s().unwrap(),
                    sets: Vec::new(),
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
        exercises[last].sets = std::mem::take(&mut sets);
    }

    if !workouts.is_empty() {
        let last = workouts.len() - 1;
        workouts[last].exercises = std::mem::take(&mut exercises);
    }

    workouts
}
