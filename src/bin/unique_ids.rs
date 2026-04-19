use anyhow::Result;

use klatsch::{Message, Node, main_loop};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

struct UniqueIdNode {
    msg_id: u32,
    node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum UniqueIdMessage {
    #[serde(rename = "generate")]
    Generate { msg_id: u32 },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum UniqueIdResponse {
    #[serde(rename = "generate_ok")]
    GenerateOk {
        msg_id: u32,
        in_reply_to: u32,
        id: Ulid,
    },
}

impl Node for UniqueIdNode {
    type M = UniqueIdMessage;

    type R = UniqueIdResponse;

    fn new(node_id: &str, _node_ids: &Vec<String>) -> Self {
        Self {
            msg_id: 0,
            node_id: node_id.to_string(),
        }
    }

    fn handle(&mut self, msg: Message<Self::M>) -> Result<()> {
        match msg.body {
            UniqueIdMessage::Generate { msg_id } => {
                let ulid = Ulid::new();

                let response = UniqueIdResponse::GenerateOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                    id: ulid,
                };

                self.msg_id += 1;

                self.send(Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                })?;

                Ok(())
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<UniqueIdNode>()?;

    Ok(())
}
