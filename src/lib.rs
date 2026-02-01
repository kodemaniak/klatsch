use std::io::stdin;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<B> {
    pub src: String,
    pub dest: String,
    pub body: B,
}

pub trait Node {
    type B: Serialize + for<'de> Deserialize<'de>;

    fn handle(&mut self, msg: Message<Self::B>) -> Message<Self::B>;
}

pub fn main_loop<N: Node>(node: &mut N) -> anyhow::Result<()> {
    for line in stdin().lines() {
        match line {
            Ok(line) => {
                eprintln!("{}", &line);
                let msg: Message<N::B> = serde_json::from_str(&line)?;
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
