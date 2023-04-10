use aws_sdk_dynamodb::{types::{AttributeValue, Select}, Client};
use lambda_http::{Request, http::StatusCode, RequestExt};
use tokio_stream::StreamExt;
use crate::common;

pub async fn get(req: Request) -> common::Result {
    let user_id = common::get_user_id(&req);
    let db = common::get_db_client();
    let query_map = req.query_string_parameters();
    let since_version = query_map.first("since");

    let result = if let Some(version) = since_version {
        let version = match version.parse() {
            Ok(v) => v,
            Err(_) => return common::empty_response(StatusCode::BAD_REQUEST),
        };
        get_changed(db, user_id, version).await
    } else {
        get_changed(db, user_id, 0).await
    };

    result
}

async fn get_changed(db: &Client, user_id: String, client_version: u64) -> common::Result {
    // Get the version first. The objects that we return may have a greater
    // modified version than this if they are modified while we're querying
    // them but that's OK. The client knows that it has at least this version
    // and possibly some pieces from a later version. The client could request
    // changes since this version to pick up the things that were modified at a
    // bad time. The version and the modified version are updated in a
    // transaction so the version will never be greater than it should be.

    let get_version = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id.clone()))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let version = get_version.item()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let collection = common::collection_from_version(version);

    // If the client is requesting changes after the current version, then we
    // know that there won't be anything so we can skip the extra queries and
    // return an empty response. If there is only one client making
    // modifications to a particular user's data, then this early-exit is the
    // path that will be taken.

    if version <= client_version {
        // TODO: since this is so common, perhaps make it smaller
        // we could use the If-None-Match header where the ETag is the version
        // number. Reaching this branch would end up being a 304 response with
        // the version in the ETag header.

        return common::json_response(StatusCode::OK, common::User {
            version,
            measurement_sets: Vec::new(),
            workouts: Vec::new(),
            exercises: Vec::new(),
            deleted_measurement_sets: Vec::new(),
            deleted_workouts: Vec::new(),
            deleted_exercises: Vec::new(),
        });
    }

    // Query for items that were modified after the given version. There's an
    // LSI on the ModifiedVersion but the index only includes the keys as to
    // avoid slowing down writes too much.

    let items = db.query()
        .table_name(common::TABLE_USER)
        .index_name(common::INDEX_MODIFIED_VERSION)
        .key_condition_expression("UserId = :userId AND ModifiedVersion > :clientVersion")
        .expression_attribute_values(":userId", AttributeValue::S(user_id.clone()))
        .expression_attribute_values(
            ":clientVersion",
            AttributeValue::N(client_version.to_string()),
        )
        .select(Select::AllAttributes)
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    // Now that we've queried for all of the items, the version is checked
    // again.

    let get_version = db.get_item()
        .table_name(common::TABLE_USER)
        .key("UserId", AttributeValue::S(user_id))
        .key("Id", AttributeValue::S("VERSION".into()))
        .send()
        .await?;

    let new_version = get_version.item()
        .map_or(0, |i| common::as_number(&i["Version"]));
    let new_collection = common::collection_from_version(new_version);

    // If the collection changed, then an import completed while we were
    // reading. So what we read was probably in the process of being deleted
    // while we were reading it. This means what we've read is probably
    // incomplete and shouldn't be trusted. We should tell the client to try
    // again.

    // If the version changed, then we have a mix of the state of the previous
    // version and the state of the new version. This is fine when fetching the
    // changes but not for exporting a full snapshot.

    if new_collection != collection {
        common::retry_later_response(0)
    } else {
        common::json_response(
            StatusCode::OK,
            common::db_to_user(version, true, true, &items),
        )
    }
}
