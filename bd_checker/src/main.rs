use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

use shared;

#[tokio::main]
async fn main() -> Result<(), Error> {
    shared::tracing::init_custom_rust_subscriber();

    lambda_runtime::run(service_fn(checker)).await?;
    Ok(())
}

async fn checker(_event: LambdaEvent<Value>) -> Result<(), Error> {
    let db_client = shared::dynamodb::BirthdaysDBClient::new().await;

    let todays_birthdays = db_client.get_todays_birthdays().await;

    if todays_birthdays.is_empty() {
        println!("No birthdays today! Maybe next time!");
    } else {
        for birthday_info in todays_birthdays {
            println!("It's {}'s birthday today! Don't forget to wish them a happy birthday!", birthday_info["fullname"].as_s().unwrap())
        }
    }

    Ok(())
}