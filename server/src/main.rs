mod common;
mod model;
mod user;
mod user_measurement;
mod user_workout;
mod user_workout_exercise;
mod user_workout_order;

use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Body, Error, Request, RequestExt, Response, request::RequestContext};

async fn api_thing_get(req: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let item = client.get_item()
        .table_name("gym-log.main")
        .key("PK", AttributeValue::S("abc".into()))
        .key("SK", AttributeValue::S("xyz".into()))
        .projection_expression("NotAReservedWord")
        .send()
        .await?;

    let value = &item.item().unwrap()["NotAReservedWord"];

    Ok(common::with_cors(Response::builder())
        .status(200)
        .header("content-type", "text/plain")
        .body(value.as_s().unwrap().as_str().into())
        .map_err(Box::new)?)
}

async fn api_thing_put(req: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let new_value = match req.into_body() {
        Body::Empty => String::new(),
        Body::Text(t) => t,
        Body::Binary(b) => String::from_utf8(b).unwrap_or(String::new()),
    };

    client.update_item()
        .table_name("gym-log.main")
        .key("PK", AttributeValue::S("abc".into()))
        .key("SK", AttributeValue::S("xyz".into()))
        .update_expression("SET NotAReservedWord = :newValue")
        .expression_attribute_values(":newValue", AttributeValue::S(new_value))
        .send()
        .await?;

    Ok(common::with_cors(Response::builder())
        .status(200)
        .body(().into())
        .map_err(Box::new)?)
}

async fn function_handler(req: Request) -> Result<Response<Body>, Error> {
    let RequestContext::ApiGatewayV2(req_ctx) = req.request_context();

    match req_ctx.route_key.as_ref().map(|s| s.as_str()) {
        Some("GET /user") => user::get(req).await,
        Some("OPTIONS /user") => common::options(),
        Some("DELETE /user/measurement/{measurementId}") => user_measurement::delete(req).await,
        Some("OPTIONS /user/measurement/{measurementId}") => common::options(),
        Some("PUT /user/measurement/{measurementId}") => user_measurement::put(req).await,
        Some("DELETE /user/workout/{workoutId}") => user_workout::delete(req).await,
        Some("OPTIONS /user/workout/{workoutId}") => common::options(),
        Some("PUT /user/workout/{workoutId}") => user_workout::put(req).await,
        Some("DELETE /user/workout/{workoutId}/exercise/{exerciseId}") => user_workout_exercise::delete(req).await,
        Some("OPTIONS /user/workout/{workoutId}/exercise/{exerciseId}") => common::options(),
        Some("PUT /user/workout/{workoutId}/exercise/{exerciseId}") => user_workout_exercise::put(req).await,
        Some("OPTIONS /user/workout/{workoutId}/order") => common::options(),
        Some("PUT /user/workout/{workoutId}/order") => user_workout_order::put(req).await,

        Some("GET /thing") => api_thing_get(req).await,
        Some("PUT /thing") => api_thing_put(req).await,
        Some("OPTIONS /thing") => common::options(),

        Some(_) | None => {
            Ok(Response::builder()
                .status(404)
                .body(().into())
                .map_err(Box::new)?)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_http::run(lambda_http::service_fn(function_handler)).await
}
