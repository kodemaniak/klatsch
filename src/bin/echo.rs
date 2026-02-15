#[derive(Debug, Default)]
struct EchoNode {
    message_ids: AtomicU32,
    node_id: String,
}

use std::sync::atomic::{AtomicU32, Ordering};

use klatsch::{Message, Node, main_loop};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum EchoMessage {
    #[serde(rename = "echo")]
    Echo { msg_id: u32, echo: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum EchoResponse {
    #[serde(rename = "echo_ok")]
    EchoOk {
        msg_id: u32,
        echo: String,
        in_reply_to: u32,
    },
}

impl Node for EchoNode {
    type M = EchoMessage;
    type R = EchoResponse;

    fn handle(&mut self, msg: Message<Self::M>) -> Message<Self::R> {
        let src = msg.src;
        match msg.body {
            EchoMessage::Echo { msg_id, echo } => {
                let id = self.message_ids.fetch_add(1, Ordering::Relaxed);
                let echo_ok = EchoResponse::EchoOk {
                    msg_id: id,
                    echo,
                    in_reply_to: msg_id,
                };
                Message {
                    src: self.node_id.clone(),
                    dest: src,
                    body: echo_ok,
                }
            }
        }
    }

    fn new(node_id: &str, _node_ids: &Vec<String>) -> Self {
        EchoNode {
            message_ids: AtomicU32::default(),
            node_id: node_id.into(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode>()?;

    Ok(())
}
