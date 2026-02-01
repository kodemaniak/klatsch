#[derive(Debug, Default)]
struct EchoNode {
    message_ids: AtomicU32,
    node_id: Option<String>,
}

use std::sync::atomic::{AtomicU32, Ordering};

use klatsch::{Message, Node, main_loop};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum EchoMessage {
    #[serde(rename = "init")]
    Init {
        msg_id: u32,
        node_id: String,
        node_ids: Vec<String>,
    },
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: u32 },
    #[serde(rename = "echo")]
    Echo { msg_id: u32, echo: String },
    #[serde(rename = "echo_ok")]
    EchoOk {
        msg_id: u32,
        echo: String,
        in_reply_to: u32,
    },
}

impl Node for EchoNode {
    type B = EchoMessage;

    fn handle(&mut self, msg: Message<Self::B>) -> Message<Self::B> {
        let src = msg.src;
        match msg.body {
            EchoMessage::Init {
                msg_id,
                node_id,
                node_ids: _,
            } if self.node_id.is_none() => {
                self.node_id = Some(node_id);
                let ok = EchoMessage::InitOk {
                    in_reply_to: msg_id,
                };

                Message {
                    src: self.node_id.as_ref().unwrap().clone(),
                    dest: src,
                    body: ok,
                }
            }
            EchoMessage::Echo { msg_id, echo } => {
                let id = self.message_ids.fetch_add(1, Ordering::Relaxed);
                let echo_ok = EchoMessage::EchoOk {
                    msg_id: id,
                    echo,
                    in_reply_to: msg_id,
                };
                Message {
                    src: self.node_id.as_ref().unwrap().clone(),
                    dest: src,
                    body: echo_ok,
                }
            }
            EchoMessage::InitOk { .. } => panic!("received init_ok, but should not"), // TODO: we should return Result
            _ => panic!("something unexpected happened"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut node = EchoNode::default();

    main_loop(&mut node)?;

    Ok(())
}
