use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Body, Error, Request, RequestExt, Response};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request

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

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(value.as_s().unwrap().as_str().into())
        .map_err(Box::new)?;
    Ok(resp)
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
