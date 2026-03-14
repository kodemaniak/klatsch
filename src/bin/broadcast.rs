use std::collections::HashMap;

use klatsch::{Message, Node, main_loop};
use serde::{Deserialize, Serialize};
use serde_json::Value;

struct BroadcastNode {
    msg_id: u32,
    node_id: String,
    topology: HashMap<String, Vec<String>>,
    messages: Vec<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum BroadcastMessage {
    #[serde(rename = "topology")]
    Topology {
        msg_id: u32,
        topology: HashMap<String, Vec<String>>,
    },
    #[serde(rename = "broadcast")]
    Broadcast { msg_id: u32, message: Value },
    #[serde(rename = "read")]
    Read { msg_id: u32 },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum BroadcastResponse {
    #[serde(rename = "topology_ok")]
    TopologyOk { msg_id: u32, in_reply_to: u32 },
    #[serde(rename = "broadcast_ok")]
    BroadcastOk { msg_id: u32, in_reply_to: u32 },
    #[serde(rename = "read_ok")]
    ReadOk {
        msg_id: u32,
        in_reply_to: u32,
        messages: Vec<Value>,
    },
}

impl Node for BroadcastNode {
    type M = BroadcastMessage;

    type R = BroadcastResponse;

    fn new(node_id: &str, _node_ids: &Vec<String>) -> Self {
        Self {
            msg_id: 0,
            node_id: node_id.to_string(),
            topology: HashMap::new(),
            messages: Vec::new(),
        }
    }

    fn handle(&mut self, msg: Message<Self::M>) -> Message<Self::R> {
        match msg.body {
            BroadcastMessage::Topology { msg_id, topology } => {
                self.topology = topology;

                let response = BroadcastResponse::TopologyOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                };

                self.msg_id += 1;

                Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                }
            }
            BroadcastMessage::Broadcast { msg_id, message } => {
                self.messages.push(message);

                let response = BroadcastResponse::BroadcastOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                };

                self.msg_id += 1;

                Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                }
            }
            BroadcastMessage::Read { msg_id } => {
                let response = BroadcastResponse::ReadOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                    messages: self.messages.clone(),
                };

                self.msg_id += 1;

                Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode>()?;

    Ok(())
}
