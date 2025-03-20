use lambda_runtime::{
    Error,
    LambdaEvent,
    service_fn,
    tracing::info
};
use serde_json::Value;

use shared::{
    dynamodb::BirthdaysDBClient,
    messaging::GMMessenger,
    tracing::init_custom_rust_subscriber,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_custom_rust_subscriber();

    lambda_runtime::run(service_fn(checker)).await?;
    Ok(())
}

async fn checker(_event: LambdaEvent<Value>) -> Result<(), Error> {
    // Gets a Vec of today's birthdays from the DynamoDB instance.
    // May be empty if there are no registered birthday's for today's date.
    let todays_birthdays = BirthdaysDBClient::new().await.get_todays_birthdays().await;

    if todays_birthdays.is_empty() {
        info!("No birthdays found for today's date, all done!");
    } else {
        let messenger = GMMessenger::new();
        for birthday in todays_birthdays {
            let fullname: String = birthday["fullname"]
                .as_s()
                .expect("Couldn't convert `fullname` to String during birthday messaging.")
                .to_owned();

            let user_id: u64 = birthday["user_id"]
                .as_n()
                .expect("Couldn't convert user_id to AttributeValue::N String during birthday messaging.")
                .parse()
                .expect("Couldn't convert user_id String to u64 during birthday messaging.");

            info!(
                "Found birthday today for {} ({}), sending birthday message...",
                fullname, user_id
            );

            messenger
                .send_message_with_mention(
                    format!(
                        "It's {}'s birthday today! Don't forget to wish them a happy birthday!",
                        fullname
                    ),
                    5,
                    fullname.len(),
                    user_id,
                )
                .await;
        }
    }

    Ok(())
}
