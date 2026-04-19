use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::{io::stdin, sync::mpsc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<B> {
    pub src: String,
    pub dest: String,
    pub body: B,
}

pub trait Node {
    type M: Serialize + for<'de> Deserialize<'de>;
    type R: Serialize;

    fn new(node_id: &str, node_ids: &Vec<String>) -> Self;

    fn handle(&mut self, msg: Message<Self::M>) -> Result<()>;

    fn send(&self, message: impl Serialize) -> Result<()> {
        let response_json = serde_json::to_string(&message)?;
        eprintln!("send: {}", response_json);
        println!("{}", response_json);

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum InitMessage {
    #[serde(rename = "init")]
    Init {
        msg_id: u32,
        node_id: String,
        node_ids: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum InitOkMessage {
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: u32 },
}

pub fn main_loop<N: Node>() -> anyhow::Result<()> {
    let mut lines = stdin().lines();

    if let Some(Ok(line)) = lines.next() {
        eprintln!("received: {}", &line);
        let init: Message<InitMessage> = serde_json::from_str(&line)?;

        let mut node: N = node_from_init(&init);

        let reply = reply_from_init(&init);
        let reply = serde_json::to_string(&reply)?;

        eprintln!("received: {}", &reply);
        println!("{}", reply);

        for line in lines {
            match line {
                Ok(line) => {
                    eprintln!("received: {}", &line);
                    let msg: Message<N::M> = serde_json::from_str(&line)?;
                    node.handle(msg)?;

                    // let response_json = serde_json::to_string(&response)?;
                    // eprintln!("response: {}", response_json);
                    // println!("{}", response_json);
                }
                Err(e) => eprintln!("error: {}", e),
            }
        }

        return Ok(());
    }

    bail!("no message received");
}

fn node_from_init<N: Node>(init: &Message<InitMessage>) -> N {
    match &init.body {
        InitMessage::Init {
            msg_id: _,
            node_id,
            node_ids,
        } => N::new(node_id, node_ids),
    }
}

fn reply_from_init(init: &Message<InitMessage>) -> Message<InitOkMessage> {
    match &init.body {
        InitMessage::Init {
            msg_id,
            node_id,
            node_ids: __,
        } => {
            let init_ok = InitOkMessage::InitOk {
                in_reply_to: *msg_id,
            };

            Message {
                src: node_id.clone(),
                dest: init.src.clone(),
                body: init_ok,
            }
        }
    }
}
