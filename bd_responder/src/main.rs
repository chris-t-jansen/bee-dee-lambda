// External dependencies
use lambda_runtime::{
    Context,
    Error,
    LambdaEvent,
    service_fn,
    tracing::info,
};
use regex::Regex;
use serde_json::Value;

// Internal dependencies
use shared::{
    dates::{day_num_to_ordinal, month_num_to_str},
    dynamodb::BirthdaysDBClient,
    messaging::GMMessenger,
    tracing::init_custom_rust_subscriber,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_custom_rust_subscriber();
    lambda_runtime::run(service_fn(responder)).await?;
    Ok(())
}

async fn responder(event: LambdaEvent<Value>) -> Result<(), Error> {
    let (event_value, _): (Value, Context) = event.into_parts();

    if !auth_valid_agent(&event_value) {
        info!("Event wasn't from an authenticated user agent, returning...");
        return Ok(());
    }

    if !check_user_sender(&event_value) {
        info!("Callback message wasn't sent by a human user, returning...");
        return Ok(());
    }

    message_handler(&event_value).await;

    Ok(())
}

async fn message_handler(event_value: &Value) {
    let msg_text: String = get_from_body_json(event_value, "text").unwrap_or_default();
    let msg_parts: (&str, &str) = msg_text.trim().split_once(' ').unwrap_or_default();

    // Matches "!bd" (case insensitive)
    let mut command_re = Regex::new(r"![Bb][Dd]").unwrap();
    if !command_re.is_match(msg_parts.0) {
        info!("Callback message isn't a command (doesn't begin with `!bd`), returning...");
        return;
    }

    // Matches "hello" or "hi" (case insensitive) with or without an exclamation point
    command_re = Regex::new(r"(([Hh][Ee][Ll]{2}[Oo])|([Hh][Ii]))!?").unwrap();
    if command_re.is_match(msg_parts.1) {
        cmd_hello_handler(event_value).await;
        return;
    }

    // Matches "me" (case insensitive)
    command_re = Regex::new(r"[Mm][Ee]").unwrap();
    if command_re.is_match(msg_parts.1) {
        cmd_me_handler(event_value).await;
        return;
    }

    // If we got to the end, then the text started with "!bd" but didn't contain a valid command phrase,
    // so send a message telling the sender that Bee-Dee couldn't understand what they were asking.
    cmd_error_handler(event_value).await;
}

async fn cmd_hello_handler(event_value: &Value) {
    let name = get_from_body_json(event_value, "name").unwrap();
    let user_id = get_from_body_json(event_value, "sender_id")
        .unwrap()
        .parse()
        .expect("Couldn't parse user's id from String to u64 in `hello` handler");

    GMMessenger::new()
        .send_message_with_mention(
            format!("Hello, {}!", name),
            7,
            name.len(),
            user_id,
        )
        .await;
}

async fn cmd_me_handler(event_value: &Value) {
    let user_id = get_from_body_json(event_value, "sender_id")
        .unwrap()
        .parse()
        .expect("Couldn't parse user's id from String to u64 in `me` handler");

    let birthday_result = BirthdaysDBClient::new()
        .await
        .get_birthday_by_user_id(user_id)
        .await;

    if birthday_result.is_none() {
        GMMessenger::new().send_message_with_mention(
            "Uh-oh, you don't have a registered birthday! Contact your local birthday-bot mechanic to have them register one for you!".to_string(),
            7,
            3,
            user_id,
        ).await;

        return;
    }

    let birthday = birthday_result.expect("Couldn't unwrap birthday query result even after `is_none` check.");

    let month_num: u8 = birthday["month_num"]
        .as_n()
        .expect("Couldn't get String from month_num AttributeValue.")
        .parse()
        .expect("Couldn't parse month_num String into u8.");

    let day_num: u8 = birthday["day_num"]
        .as_n()
        .expect("Couldn't get String from day_num AttributeValue.")
        .parse()
        .expect("Couldn't parse day_num String into u8.");

    GMMessenger::new()
        .send_message_with_mention(
            format!(
                "Your birthday is registered as {} {}!\nIf that isn't right, contact your local birthday-bot mechanic to have it corrected!",
                month_num_to_str(month_num),
                day_num_to_ordinal(day_num)
            ),
            0,
            4,
            user_id,
        )
        .await;
}

async fn cmd_error_handler(event_value: &Value) {
    let user_id = get_from_body_json(event_value, "sender_id")
        .unwrap()
        .parse()
        .expect("Couldn't parse user's id from String to u64 in error handler");

    GMMessenger::new()
        .send_message_with_mention(
            "Uh-oh, looks like you were trying to ask me something, but I'm not sure what! Check your spelling and try again.".to_string(),
            18,
            3,
            user_id,
        )
        .await;
}

fn auth_valid_agent(event_value: &Value) -> bool {
    // Gets the user agent text from the payload JSON
    let user_agent: &str = event_value["requestContext"]["http"]["userAgent"]
        .as_str()
        .unwrap_or("");

    info!("Found userAgent `{}`, comparing to `GroupMeBotNotifier`...", user_agent);

    // Checks if the user agent text matches the expected text, returning the result
    user_agent.starts_with("GroupMeBotNotifier")
}

fn check_user_sender(event_value: &Value) -> bool {
    let sender_type_result = get_from_body_json(event_value, "sender_type");

    if let Some(sender_type) = sender_type_result {
        info!("Found sender_type `{}`, comparing to `user`...", sender_type);
        return sender_type == "user";
    } else {
        return false;
    }
}

fn get_from_body_json(event_value: &Value, key: &str) -> Option<String> {
    let body_json: Value =
        serde_json::from_str::<Value>(event_value["body"].as_str().unwrap_or(""))
            .expect("Couldn't convert `body` response string to JSON.");

    if let Some(value) = body_json.get(key) {
        let mut string_value = value.to_string();
        string_value.retain(|c| c != '"');
        return Some(string_value);
    } else {
        return None;
    }
}
