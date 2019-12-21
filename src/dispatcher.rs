use tokio::net::udp::SendHalf;
use tokio::sync::mpsc::Receiver;

use crate::types::*;

pub struct Dispatcher {
    pub chan: Receiver<Bytes>,
    pub sock: SendHalf,
}

impl Dispatcher {
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        let Dispatcher{sock, chan} = self;
        while let Some(b) = chan.recv().await {
            sock.send(&b).await;
        };
        Ok(())
    }
}
