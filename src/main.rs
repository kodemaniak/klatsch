use std::{
    io::stdin,
    sync::atomic::{AtomicU32, Ordering},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message<B> {
    src: String,
    dest: String,
    body: B,
}

trait Node {
    type B;

    fn handle(&mut self, msg: Message<Self::B>) -> Message<Self::B>;
}

#[derive(Debug, Default)]
struct EchoNode {
    message_ids: AtomicU32,
    node_id: Option<String>,
}

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

    for line in stdin().lines() {
        match line {
            Ok(line) => {
                eprintln!("{}", &line);
                let msg: Message<EchoMessage> = serde_json::from_str(&line)?;
                let response = node.handle(msg);
                let response_json = serde_json::to_string(&response)?;
                eprintln!("{}", &response_json);
                println!("{}", response_json);
            }
            Err(e) => eprintln!("error: {}", e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{EchoMessage, Message};

    #[test]
    pub fn test_deser() {
        let msg_string = r#"
            {"id":0,"src":"c0","dest":"n2","body":{"type":"init","node_id":"n2","node_ids":["n1","n2","n3","n4","n5"],"msg_id":1}}
        "#;
        let _: Message<EchoMessage> = serde_json::from_str(&msg_string).unwrap();
    }
}
