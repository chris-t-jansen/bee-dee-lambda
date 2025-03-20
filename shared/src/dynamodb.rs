use std::collections::HashMap;

use aws_config;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::{Datelike, Local};

use crate::secrets;

#[derive(Debug)]
pub struct BirthdaysDBClient {
    client: Client
}

impl BirthdaysDBClient {
    pub async fn new() -> Self {
        BirthdaysDBClient {
            client: Client::new(&aws_config::load_from_env().await)
        }
    }

    pub async fn get_todays_birthdays(&self) -> Vec<HashMap<String, AttributeValue>> {
        let current_day: u32 = Local::now().day();
        let current_month: u32 = Local::now().month();

        // Equivalent to an SQL query along the lines of:
        //      SELECT * FROM table_arn WHERE month_num = current_month AND day_num = current_day
        let query = self.client.query()
            .table_name(secrets::BIRTHDAYS_DB_ARN)
            .index_name("month-day-index")
            .key_condition_expression("#mon = :monnum and #day = :daynum")
            .expression_attribute_names("#mon", "month_num")
            .expression_attribute_values(":monnum", AttributeValue::N(current_month.to_string()))
            .expression_attribute_names("#day", "day_num")
            .expression_attribute_values(":daynum", AttributeValue::N(current_day.to_string()));

        // Sends the query and awaits the result
        let results = query.send().await.expect("Couldn't get `month-day` query results from DynamoDB instance!");

        // Returns all the birthdays for today, or an empty vector if there are none
        results.items.unwrap_or(Vec::new())
    }

    pub async fn get_birthday_by_user_id(&self, user_id: u64) -> Option<HashMap<String, AttributeValue>> {
        // Equivelent to an SQL query along the lines of:
        //      SELECT * FROM table_arn WHERE user_id = user_id
        let query = self.client.query()
            .table_name(secrets::BIRTHDAYS_DB_ARN)
            .key_condition_expression("#userid = :userid")
            .expression_attribute_names("#userid", "user_id")
            .expression_attribute_values(":userid", AttributeValue::N(user_id.to_string()));

        // Sends the query and awaits the result
        let results = query.send().await.expect("Couldn't get `user_id` query results from DynamoDB instance!");

        // Since user_id is the partition key, the query should only return an empty Vec (no results)
        // or a Vec with a single item (one result). Therefore, this function just returns the first
        // result if it exists or None if it doesn't.
        if results.items.is_some() {
            return Some(results.items.expect("Failed to unwrap DynamoDB query results even after `is_some()` check!")[0].clone());
        } else {
            return None;
        }
    }
}