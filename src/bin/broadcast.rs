use std::{collections::HashMap, sync::mpsc};

use anyhow::Result;
use klatsch::{Message, Node, main_loop};
use serde::{Deserialize, Serialize};
use serde_json::Value;

struct BroadcastNode {
    msg_id: u32,
    node_id: String,
    messages: Vec<Value>,
    node_messages: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    #[serde(rename = "gossip")]
    Gossip { msg_id: u32, message: Value },
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
            messages: Vec::new(),
            node_messages: HashMap::new(),
        }
    }

    fn handle(&mut self, msg: Message<Self::M>) -> Result<()> {
        match msg.body {
            BroadcastMessage::Topology { msg_id, topology } => {
                let neighbors = topology
                    .get(&self.node_id)
                    .expect("own node_id must be part of topology");

                self.node_messages.retain(|k, _| neighbors.contains(k));

                for node in neighbors {
                    if self.node_messages.contains_key(node) {
                        continue;
                    }

                    self.node_messages.insert(node.clone(), 0);
                }

                let response = BroadcastResponse::TopologyOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                };

                self.msg_id += 1;
                eprintln!("neighbors: {:?}", &neighbors);
                eprintln!("topology: {:?}", &topology);

                eprintln!("topology_own: {:?}", &self.node_messages);

                self.send(Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                })?;

                Ok(())
            }
            BroadcastMessage::Broadcast { msg_id, message } => {
                // TODO: get rid of clone
                let topology_nodes = self.node_messages.clone();
                let topology_nodes = topology_nodes.keys();
                for node_id in topology_nodes.filter(|nid| **nid != msg.src) {
                    let bcast = BroadcastMessage::Gossip {
                        msg_id: self.msg_id,
                        message: message.clone(),
                    };
                    let bmsg = Message {
                        src: self.node_id.clone(),
                        dest: node_id.to_owned(),
                        body: bcast,
                    };
                    self.send(bmsg).unwrap();

                    self.msg_id += 1;

                    let old_msg_id = self.node_messages.entry(node_id.clone()).or_default();
                    *old_msg_id = msg_id;
                }

                self.messages.push(message.clone());

                eprintln!("message: {:?}", &self.messages);

                let response = BroadcastResponse::BroadcastOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                };

                self.msg_id += 1;

                self.send(Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                })?;

                Ok(())
            }
            BroadcastMessage::Read { msg_id } => {
                let response = BroadcastResponse::ReadOk {
                    msg_id: self.msg_id,
                    in_reply_to: msg_id,
                    messages: self.messages.clone(),
                };

                self.msg_id += 1;

                self.send(Message {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: response,
                })?;

                Ok(())
            }
            BroadcastMessage::Gossip { msg_id: _, message } => {
                // TODO: get rid of clone
                if !self.messages.contains(&message) {
                    for node_id in self.node_messages.keys() {
                        let bcast = BroadcastMessage::Gossip {
                            msg_id: self.msg_id,
                            message: message.clone(),
                        };
                        let bmsg = Message {
                            src: self.node_id.clone(),
                            dest: node_id.to_owned(),
                            body: bcast,
                        };
                        self.send(bmsg).unwrap();

                        self.msg_id += 1;
                    }

                    self.messages.push(message.clone());

                    eprintln!("messages: {:?}", &self.messages);
                }

                Ok(())
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode>()?;

    Ok(())
}
