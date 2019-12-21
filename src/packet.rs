use std::io::Error;

use tokio::sync::mpsc::{Receiver, channel};

use uuid::Uuid;

use crate::client::Client;
use crate::types::*;
use crate::errors::*;

#[derive(Clone)]
pub struct Packet {
    pub cont: Bytes,
    pub uuid: Uuid,
}

impl Packet {
    pub async fn dispatch(self,  c: &mut Client) -> Result<Receiver<Result<Bytes, Error>>, std::io::Error> {
        let Packet{cont, uuid} = self;
        let Client{chan, tabl} = c;
        let (tx, rx) = channel(1024 * 1024 * 1024);
        let mut tabl = tabl.lock().await;
        tabl.insert(uuid, tx);
        let _ = chan.send(cont).await;
        Ok(rx)
    }
}
