pub mod handler;
pub mod payload;

use crate::payload::ParsingError::{InvalidPayload, InvalidTimestamp};
use crate::payload::AlicaMessagePayload;

impl AlicaMessagePayload {
    // payload syntax: agent_id|message_type|message|timestamp
    pub fn from(bytes: Vec<u8>) -> Result<AlicaMessagePayload, payload::ParsingError> {
        let payload = String::from_utf8(bytes)
            .map_err(|_| { InvalidPayload("Payload is no string".to_string()) })?;

        let mut content = payload.split("|");
        let part_count = content.clone().count();

        if part_count != 4 {
            Err(InvalidPayload("Payload needs to have exactly 4 parts".to_string()))
        } else {
            let agent_id = content.next().unwrap().to_string();
            let message_type = content.next().unwrap().to_string();
            let message_bytes = content.next().unwrap().as_bytes().to_vec();
            let timestamp = content.next().unwrap().parse::<u64>()
                .map_err(|_| InvalidTimestamp)?;

            Ok(AlicaMessagePayload {
                agent_id,
                message_type,
                message_bytes,
                timestamp,
            })
        }
    }
}
