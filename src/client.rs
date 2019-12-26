
use std::sync::Arc;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use futures::future::join;

use tokio::net::{UdpSocket, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::time::{delay_for, Duration};

use crate::dispatcher::*;
use crate::listener::*;
use crate::request::*;
use crate::types::*;

/// `Client` has `Sender<Bytes>` for sending bytes
/// and `Arc<Mutex<Table>>` which has status of sended packets.
#[derive(Clone)]
pub struct Client {
    pub chan: Sender<Bytes>,
    pub tabl: Arc<Mutex<Table>>,
}

impl Client {
    /// Open a connection to a twinkle server.
    pub async fn open<A: ToSocketAddrs>(addr: A) -> Result<(Client, Manager), std::io::Error> {
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        sock.connect(addr).await?;
        let (txc, rxc) = channel(1024*1024*1024);

        let tabl = Arc::new(Mutex::new(HashMap::new()));
        let c = Client{chan: txc, tabl: tabl.clone()};
        let m = Manager::new(rxc, sock, tabl);
        Ok((c, m))
    }

    /// Send `Ping`.
    pub async fn ping(&mut self) -> Result<Bytes, std::io::Error> {
        send(self, Request::Ping).await
    }
    /// Send `Get`.
    pub async fn get(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        send(self, Request::Get(key)).await
    }
    /// Send `Set`.
    pub async fn set(&mut self, key: Bytes, val: Bytes) -> Result<Bytes, std::io::Error> {
        send(self, Request::Set(key, val)).await
    }
    /// Send `Unset`.
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

/// `Manager` manages to send packets and to receive packets.
pub struct Manager {
    dispatcher: Dispatcher,
    listener: Listener,
}

impl Manager {
    fn new(chan: Receiver<Bytes>, sock: UdpSocket, tabl: Arc<Mutex<Table>>) -> Manager{
        let (rxs, txs) = sock.split();
        Manager{
            dispatcher: Dispatcher{chan, sock: txs},
            listener: Listener{sock: rxs, tabl}
        }
    }
    /// Run the `Manager`.
    pub async fn run(mut self) -> Result<(), std::io::Error>{
        let _ = join(
            self.dispatcher.run(),
            self.listener.run()
        ).await;
        Ok(())
    }
}
