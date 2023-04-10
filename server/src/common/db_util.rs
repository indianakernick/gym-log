use aws_sdk_dynamodb::{
    Client,
    error::SdkError,
    operation::transact_write_items::TransactWriteItemsError,
    types::{AttributeValue, WriteRequest, CancellationReason},
};
use lambda_http::Error;
use once_cell::sync::OnceCell;

static CLIENT: OnceCell<Client> = OnceCell::new();

pub fn get_db_client() -> &'static Client {
    CLIENT.get().unwrap()
}

pub async fn init_db_client() {
    let config = aws_config::load_from_env().await;
    CLIENT.set(Client::new(&config)).unwrap();
}

pub fn as_number<N>(attribute: &AttributeValue) -> N
    where
        N: std::str::FromStr,
        <N as std::str::FromStr>::Err: std::fmt::Debug,
{
    attribute.as_n().unwrap().parse().unwrap()
}

pub async fn batch_write(
    db: &Client,
    table: &str,
    requests: Vec<WriteRequest>,
) -> Result<(), Error> {
    const MAX_BATCH_SIZE: usize = 25;

    let mut unprocessed = Vec::new();
    let mut processed = 0;

    while !unprocessed.is_empty() || processed < requests.len() {
        let mut batch;
        let remaining = requests.len() - processed;

        if !unprocessed.is_empty() {
            batch = unprocessed;
            let processing = remaining.min(MAX_BATCH_SIZE - batch.len());
            batch.extend_from_slice(&requests[processed..processed + processing]);
            processed += processing;
        } else {
            let processing = remaining.min(MAX_BATCH_SIZE);
            batch = Vec::from(&requests[processed..processed + processing]);
            processed += processing;
        }

        let batch_write = db.batch_write_item()
            .request_items(table, batch)
            .send()
            .await?;

        unprocessed = batch_write.unprocessed_items()
            .and_then(|map| map.get(table))
            .cloned()
            .unwrap_or_default();
    }

    Ok(())
}

pub fn transact_write_cancellation_reasons(
    error: &SdkError<TransactWriteItemsError>,
) -> Option<&[CancellationReason]> {
    if let SdkError::ServiceError(service_error) = error {
        if let TransactWriteItemsError::TransactionCanceledException(cancel) = &service_error.err() {
            return cancel.cancellation_reasons();
        }
    }
    None
}
