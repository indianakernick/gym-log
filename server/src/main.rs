use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Body, Error, Request, RequestExt, Response, request::RequestContext};

async fn api_thing_get(event: Request) -> Result<Response<Body>, Error> {
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

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(value.as_s().unwrap().as_str().into())
        .map_err(Box::new)?)
}

async fn api_thing_put(event: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let new_value = match event.into_body() {
        Body::Empty => String::new(),
        Body::Text(t) => t,
        Body::Binary(b) => String::from_utf8(b).unwrap_or(String::new()),
    };

    client.update_item()
        .table_name("gym-log.main")
        .key("PK", AttributeValue::S("abc".into()))
        .key("SK", AttributeValue::S("xyz".into()))
        .update_expression("SET NotAReservedWord = :newValue")
        .expression_attribute_values("newValue", AttributeValue::S(new_value))
        .send()
        .await?;

    Ok(Response::builder()
        .status(201)
        .body(().into())
        .map_err(Box::new)?)
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let RequestContext::ApiGatewayV2(req_ctx) = event.request_context();

    match req_ctx.route_key.as_ref().map(|s| s.as_str()) {
        Some("GET /thing") => api_thing_get(event).await,
        Some("PUT /thing") => api_thing_put(event).await,
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
