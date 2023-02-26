mod common;
mod handlers;

use lambda_http::{Body, Error, Request, RequestExt, Response, request::RequestContext, http::StatusCode};

async fn function_handler(req: Request) -> Result<Response<Body>, Error> {
    use handlers::*;

    let RequestContext::ApiGatewayV2(req_ctx) = req.request_context();

    match req_ctx.route_key.as_deref() {
        Some("GET /user") => user::get(req).await,
        Some("DELETE /user/measurement/{measurementId}") => user_measurement::delete(req).await,
        Some("PUT /user/measurement/{measurementId}") => user_measurement::put(req).await,
        Some("DELETE /user/workout/{workoutId}") => user_workout::delete(req).await,
        Some("PUT /user/workout/{workoutId}") => user_workout::put(req).await,
        Some("DELETE /user/workout/{workoutId}/exercise/{exerciseId}") => user_workout_exercise::delete(req).await,
        Some("PUT /user/workout/{workoutId}/exercise/{exerciseId}") => user_workout_exercise::put(req).await,
        Some("PUT /user/workout/{workoutId}/order") => user_workout_order::put(req).await,

        Some(_) | None => common::empty_response(StatusCode::NOT_FOUND)
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

    common::init_db_client().await;

    lambda_http::run(lambda_http::service_fn(function_handler)).await
}
