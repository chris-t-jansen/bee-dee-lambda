use reqwest;
use serde_json::json;

use crate::secrets;

pub struct GMMessenger {
    client: reqwest::Client
}

impl GMMessenger {
    pub fn new() -> Self {
        GMMessenger {
            client: reqwest::Client::new()
        }
    }

    pub async fn send_message_with_mention(&self, msg_text: String, men_loci_start_pos: usize, men_loci_len: usize, men_id: u64) {
        let msg_body = json!({
            "bot_id": secrets::GROUPME_BOT_ID,
            "text": msg_text,
            "attachments": [
                {
                    "type": "mentions",
                    "user_ids": [
                        men_id
                    ],
                    "loci": [
                        [ men_loci_start_pos, men_loci_len ]
                    ],
                }
            ]
        });
    
        self.client.post(secrets::GROUPME_POST_URL)
            .body(msg_body.to_string())
            .send()
            .await
            .expect("Failed to send GroupMe message with mention.");
    }
}