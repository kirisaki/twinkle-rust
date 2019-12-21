use std::sync::Arc;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use tokio::net::{UdpSocket, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Sender, channel};
use tokio::time::{delay_for, Duration};

use crate::dispatcher::*;
use crate::listener::*;
use crate::request::*;
use crate::types::*;

#[derive(Clone)]
pub struct Client {
    pub chan: Sender<Bytes>,
    pub tabl: Arc<Mutex<Table>>,
}

impl Client {
    pub async fn open<A: ToSocketAddrs>(addr: A) -> Result<(Client, Dispatcher, Listener), std::io::Error> {
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        sock.connect(addr).await?;
        let (rxs, txs) = sock.split();
        let (txc, rxc) = channel(1024*1024*1024);

        let tabl = Arc::new(Mutex::new(HashMap::new()));
        let c = Client{chan: txc, tabl: tabl.clone()};
        let d = Dispatcher{chan: rxc, sock: txs};
        let l = Listener{sock: rxs, tabl: tabl.clone()};
        Ok((c, d, l))
    }

    pub async fn ping(&mut self) -> Result<Bytes, std::io::Error> {
        send(self, Request::Ping).await
    }
    pub async fn get(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        send(self, Request::Get(key)).await
    }
    pub async fn set(&mut self, key: Bytes, val: Bytes) -> Result<Bytes, std::io::Error> {
        send(self, Request::Set(key, val)).await
    }
    pub async fn unset(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        send(self, Request::Unset(key)).await
    }
}

async fn send(client: &mut Client, request: Request) -> Result<Bytes, std::io::Error>{
    let packet = request.issue()?;
    for _ in 0..10 {
        let mut rx = packet.clone().dispatch(client).await?;
        for i in 0..5u32 {
            delay_for(Duration::from_millis(2u64.pow(i))).await;
            match rx.try_recv() {
                Ok(v) => return v,
                Err(_) => {},
            };

        };
    }
    Err(Error::new(ErrorKind::Other, "request timeout"))
}
