use anyhow::{Context, Ok};
use rustengan::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    node: String,
    id: usize,
    messages: Vec<usize>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_init_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            id: 1,
            node: init.node_id,
            messages: Vec::new(),
        })
    }
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Broadcast { message } => {
                self.messages.push(message);
                reply.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *output, &reply)
                    .context("serializer response to echo")?;
                output.write_all(b"\n").context("write new line")?;
                self.id += 1;
            }
            Payload::Read => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serializer response to echo")?;
                output.write_all(b"\n").context("write new line")?;
                self.id += 1;
            }
            Payload::Topology { topology: _ } => {
                reply.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *output, &reply)
                    .context("serializer response to echo")?;
                output.write_all(b"\n").context("write new line")?;
                self.id += 1;
            }
            Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
