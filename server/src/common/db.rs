use aws_sdk_dynamodb::types::AttributeValue;
use once_cell::sync::OnceCell;

static CLIENT: OnceCell<aws_sdk_dynamodb::Client> = OnceCell::new();

pub fn get_db_client() -> &'static aws_sdk_dynamodb::Client {
    CLIENT.get().unwrap()
}

pub async fn init_db_client() {
    let config = aws_config::load_from_env().await;
    CLIENT.set(aws_sdk_dynamodb::Client::new(&config)).unwrap();
}

pub fn as_number<N>(attribute: &AttributeValue) -> N
    where
        N: std::str::FromStr,
        <N as std::str::FromStr>::Err: std::fmt::Debug,
{
    attribute.as_n().unwrap().parse().unwrap()
}
