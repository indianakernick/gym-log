use lambda_http::{Request, RequestExt, http::StatusCode};
use crate::common;

pub async fn delete(req: Request) -> common::Result {
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if !common::is_date(measurement_id) {
        return common::empty_response(StatusCode::NOT_FOUND);
    }

    common::version_delete::<common::MeasurementSet>(&req, measurement_id).await
}

pub async fn put(req: Request) -> common::Result {
    let params = req.path_parameters();
    let measurement_id = params.first("measurementId").unwrap();

    if let Err(e) = common::validate_date(measurement_id) {
        return e;
    }

    common::version_modify(
        &req,
        common::version_put_item::<common::MeasurementSet>(measurement_id)
    ).await
}
