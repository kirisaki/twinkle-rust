use std::sync::Arc;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use tokio::net::{UdpSocket, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Sender, channel};
use tokio::net::udp::SendHalf;
use tokio::time::{timeout, delay_for, Duration};

use crate::dispatcher::*;
use crate::listener::*;
use crate::request::*;
use crate::types::*;
use crate::packet::*;

#[derive(Clone)]
pub struct Client {
    pub chan: Sender<Bytes>,
    pub tabl: Arc<Mutex<Table>>,
}

impl Client {
    pub async fn open<A: ToSocketAddrs>(addr: A) -> Result<(Client, Dispatcher, Listener), std::io::Error> {
        let sock = UdpSocket::bind("127.0.0.1:0").await?;
        sock.connect(addr).await?;
        let (rxs, txs) = sock.split();
        let (txc, rxc) = channel(1024);

        let tabl = Arc::new(Mutex::new(HashMap::new()));
        let c = Client{chan: txc, tabl: tabl.clone()};
        let d = Dispatcher{chan: rxc, sock: txs};
        let l = Listener{sock: rxs, tabl: tabl.clone()};
        Ok((c, d, l))
    }

    pub async fn ping(&mut self) -> Result<(), std::io::Error> {
        let mut rx = Request::Ping.issue()?.dispatch(self).await?;
        let _ = timeout(TIMEOUT, rx.recv()).await?;
        Ok(())
    }
    pub async fn get(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        let packet = Request::Get(key).issue()?;
        for i in 0..10u64 {
            let mut rx = packet.clone().dispatch(self).await?;
            match rx.try_recv() {
                Ok(v) => return v,
                Err(_) => {},
            };
            delay_for(Duration::from_nanos(i)).await;
        };
        Err(Error::new(ErrorKind::Other, "request timeout"))
    }
    pub async fn set(&mut self, key: Bytes, val: Bytes) -> Result<(), std::io::Error> {
        let packet = Request::Set(key, val).issue()?;
        for i in 0..10u64 {
            let mut rx = packet.clone().dispatch(self).await?;
            match rx.try_recv() {
                Ok(_) => return Ok(()),
                Err(_) => {},
            };
            delay_for(Duration::from_nanos(i)).await;
        };
        Err(Error::new(ErrorKind::Other, "request timeout"))
    }
    pub async fn unset(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        let mut rx = Request::Unset(key).issue()?.dispatch(self).await?;
        match rx.recv().await {
            Some(v) => v,
            None => Err(Error::new(ErrorKind::Other, "key notfound")),
        }
    }
}

