use std::ops::ControlFlow;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn delete(req: Request) -> common::Result {
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if !common::is_date(measurement_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    common::version_delete(&req, format!("MEASUREMENT#{measurement_id}")).await
}

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if let Err(e) = common::validate_date(measurement_id) {
        return e;
    }

    let body = match common::parse_request_json::<common::VersionModifyReq<_>>(&req) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let collection_prefix = common::get_collection_prefix(
        common::collection_from_version(body.version)
    );

    common::version_apply(
        &req,
        body.version,
        |builder, user_id, new_version| {
            common::version_put_item(
                format!("{collection_prefix}MEASUREMENT#{measurement_id}"),
                |builder, item: common::MeasurementSet| {
                    builder
                        .item("Notes", AttributeValue::S(item.notes.0.into()))
                        .item("Measurements", AttributeValue::M(
                            item.measurements.iter()
                                .map(|(k, v)| ((*k).into(), AttributeValue::N(v.to_string())))
                                .collect()
                        ))
                }
            )(builder, body.item, user_id, new_version)
        },
        |_| ControlFlow::Continue(()),
    ).await
}
