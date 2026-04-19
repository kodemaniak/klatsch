use anyhow::Result;

use klatsch::{Message, Node, main_loop};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
struct EchoNode {
    msg_id: u32,
    node_id: String,
}

#[derive(Serialize, Debug, Deserialize)]
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

    fn handle(&mut self, msg: Message<Self::M>) -> Result<()> {
        let src = msg.src;
        match msg.body {
            EchoMessage::Echo { msg_id, echo } => {
                let echo_ok = EchoResponse::EchoOk {
                    msg_id: self.msg_id,
                    echo,
                    in_reply_to: msg_id,
                };

                self.msg_id += 1;

                self.send(Message {
                    src: self.node_id.clone(),
                    dest: src,
                    body: echo_ok,
                })?;

                Ok(())
            }
        }
    }

    fn new(node_id: &str, _node_ids: &Vec<String>) -> Self {
        EchoNode {
            msg_id: 0,
            node_id: node_id.into(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode>()?;

    Ok(())
}
