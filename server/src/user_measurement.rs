use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::{Request, RequestExt, http::StatusCode};
use super::{common, model};

pub async fn delete(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if !common::is_uuid(measurement_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    let db = common::get_db_client().await;

    let result = db.delete_item()
        .table_name(common::TABLE_USER_MEASUREMENT)
        .key("UserId", AttributeValue::S(user_id))
        .key("MeasurementId", AttributeValue::S(measurement_id.into()))
        .condition_expression("attribute_exists(UserId)")
        .send()
        .await;

    common::delete_response(result)
}

pub async fn put(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if let Err(e) = common::validate_uuid(measurement_id) {
        return e;
    }

    let measurement = match common::parse_request_json::<model::Measurement>(&req) {
        Ok(m) => m,
        Err(e) => return e,
    };

    if let Err(e) = chrono::NaiveDate::parse_from_str(measurement.capture_date, "%F") {
        return common::error_response(StatusCode::BAD_REQUEST, &format!("Invalid capture_date: {}", e));
    }

    let db = common::get_db_client().await;

    db.put_item()
        .table_name(common::TABLE_USER_MEASUREMENT)
        .item("UserId", AttributeValue::S(user_id))
        .item("MeasurementId", AttributeValue::S(measurement_id.into()))
        .item("MeasurementType", AttributeValue::S(measurement.r#type.into()))
        .item("CaptureDate", AttributeValue::S(measurement.capture_date.into()))
        .item("Value", AttributeValue::N(measurement.value.to_string()))
        .item("Notes", AttributeValue::S(measurement.notes.into()))
        .send()
        .await?;

    // Should we return a 201 if an item was created?
    // Do we care?

    common::empty_response(StatusCode::OK)
}
