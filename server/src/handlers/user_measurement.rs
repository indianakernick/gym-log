use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn delete(req: Request) -> common::Result {
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if !common::is_uuid(measurement_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    common::version_delete(&req, format!("MEASUREMENT#{measurement_id}")).await
}

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if let Err(e) = common::validate_uuid(measurement_id) {
        return e;
    }

    common::version_modify(&req, common::version_put_item(
        format!("MEASUREMENT#{measurement_id}"),
        |builder, item: &common::Measurement| {
            builder
                .item("Type", AttributeValue::S(item.r#type.into()))
                .item("CaptureDate", AttributeValue::S(item.capture_date.into()))
                .item("Value", AttributeValue::N(item.value.to_string()))
                .item("Notes", AttributeValue::S(item.notes.into()))
        }
    )).await
}
