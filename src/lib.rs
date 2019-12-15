use tokio::net::{UdpSocket, ToSocketAddrs};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use futures::future;
use tokio::sync::Mutex;
use std::sync::{Arc};
use tokio::net::udp::{RecvHalf, SendHalf};
use tokio::time::{timeout};
use std::time::{Duration};
use std::io::{Error, ErrorKind};

// limitation of UDP
const BUF_SIZE: usize = 64 * 1024;
const UUID_LEN: usize = 16;
const TIMEOUT: Duration = Duration::from_secs(1);

type Bytes = Vec<u8>;
type Table = HashMap<Uuid, Sender<Result<Bytes, Error>>>;

pub enum TwinkleError {
    Failed,
}

pub struct Client {
    sock: SendHalf,
    tabl: Arc<Mutex<Table>>,
}

pub struct Listener {
    sock: RecvHalf,
    tabl: Arc<Mutex<Table>>,
}

#[derive(Debug, PartialEq)]
pub enum Request {
    Ping,
    Get(Bytes),
    Set(Bytes, Bytes),
    Unset(Bytes),
}

pub async fn open<A: ToSocketAddrs>(addr: A) -> Result<(Client, Listener), std::io::Error> {
    let sock = UdpSocket::bind("127.0.0.1:0").await?;
    sock.connect(addr).await?;
    let (rx, tx) = sock.split();

    let tabl = Arc::new(Mutex::new(HashMap::new()));
    let c = Client{sock: tx, tabl: tabl.clone()};
    let l = Listener{sock: rx, tabl: tabl.clone()};
    Ok((c, l))
}


impl Client {
    pub async fn ping(&mut self) -> Result<(), std::io::Error> {
        let mut rx = Request::Ping.issue()?.dispatch(self).await?;
        let _ = timeout(TIMEOUT, rx.recv()).await?;
        Ok(())
    }
    pub async fn get(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        let mut rx = Request::Get(key).issue()?.dispatch(self).await?;
        match rx.recv().await {
            Some(v) => v,
            None => Err(Error::new(ErrorKind::Other, "key notfound")),
        }
    }
    pub async fn set(&mut self, key: Bytes, val: Bytes) -> Result<Bytes, std::io::Error> {
        let mut rx = Request::Set(key, val).issue()?.dispatch(self).await?;
        match rx.recv().await {
            Some(v) => v,
            None => Err(Error::new(ErrorKind::Other, "key notfound")),
        }
    }
    pub async fn unset(&mut self, key: Bytes) -> Result<Bytes, std::io::Error> {
        let mut rx = Request::Unset(key).issue()?.dispatch(self).await?;
        match rx.recv().await {
            Some(v) => v,
            None => Err(Error::new(ErrorKind::Other, "key notfound")),
        }
    }
}

impl Listener {
    pub async fn listen(&mut self) -> Result<(), std::io::Error> {
        let Listener{sock, tabl} = self;
        let mut buf = vec![0; BUF_SIZE];
        loop {
            let amt = sock.recv(&mut buf).await?;
            let e = std::io::Error::new(std::io::ErrorKind::Other, "failed parsing");
            if amt < UUID_LEN + 1 {
                return Err(e);
            } else {
                let uuid = match Uuid::from_slice(&buf[1..UUID_LEN+1]) {
                    Ok(v) => v,
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
                };
                let mut tabl = tabl.lock().await;
                let val = if UUID_LEN +1 == amt {
                    vec![]
                } else {
                    buf[UUID_LEN + 2..amt].to_vec()
                };
                match tabl.get_mut(&uuid) {
                    Some(c) => {
                        if buf[0] == 0x01 {
                            let _ = c.send(Ok(val)).await;
                        } else {
                            let _ = c.send(Err(Error::new(ErrorKind::Other, "command failed"))).await;
                        };
                    },
                    None => return Err(e),
                };
                tabl.remove(&uuid);
            }

        }
    }
}


impl Request {
    pub fn issue(self) -> Result<(Packet), std::io::Error> {
        let uuid = Uuid::new_v4();
        let cont = match self {
            Request::Ping => {
                let mut c = vec![0x01];
                c.append(&mut uuid.as_bytes().to_vec());
                c
            },
            Request::Get(key) => {
                let ks = usize_to_bytes(key.len())?;
                let mut c = vec![0x02];
                c.append(&mut uuid.as_bytes().to_vec());
                c.append(&mut ks.to_vec());
                c.append(&mut key.to_vec());
                c
            },
            Request::Set(key, val) => {
                let ks = usize_to_bytes(key.len())?;
                let mut c = vec![0x03];
                c.append(&mut uuid.as_bytes().to_vec());
                c.append(&mut ks.to_vec());
                c.append(&mut key.to_vec());
                c.append(&mut val.to_vec());
                c
            },
            Request::Unset(key) => {
                let ks = usize_to_bytes(key.len())?;
                let mut c = vec![0x04];
                c.append(&mut uuid.as_bytes().to_vec());
                c.append(&mut ks.to_vec());
                c.append(&mut key.to_vec());
                c
            },
        };
        Ok(Packet{cont, uuid})
    }
}

fn usize_to_bytes(u: usize) -> Result<[u8; 2], std::io::Error> {
    if u > From::from(u16::max_value()){
        Err(std::io::Error::new(std::io::ErrorKind::Other, "keylen too long"))
    } else {
        let ks = u.to_be_bytes();
        Ok([ks[6], ks[7]])
    }
}

pub struct Packet {
    cont: Bytes,
    uuid: Uuid,
}

impl Packet {
    pub async fn dispatch(self,  c: &mut Client) -> Result<Receiver<Result<Bytes, Error>>, std::io::Error> {
        let Packet{cont, uuid} = self;
        let Client{sock, tabl} = c;
        let (tx, rx) = channel(1024 * 64);
        let mut tabl = tabl.lock().await;
        tabl.insert(uuid, tx);
        sock.send(&cont).await?;
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    const WAIT: Duration = std::time::Duration::from_secs(1);

    #[tokio::test]
    async fn test_ping() -> Result<(), std::io::Error> {
        let (mut client, mut listener) = open("127.0.0.1:3000".to_string()).await?;
        let (_ ,res) = future::join(timeout(WAIT ,listener.listen()), client.ping()).await;
        res
    }

    #[tokio::test]
    async fn test_get_not_found() -> Result<(), std::io::Error> {
        let (mut client, mut listener) = open("127.0.0.1:3000".to_string()).await?;
        let (_, res) = future::join(timeout(WAIT ,listener.listen()), timeout(WAIT, async move {
            client.unset(b"hoge".to_vec()).await;
            client.get(b"hoge".to_vec()).await
        })).await;
        match res.unwrap() {
            Ok(_) => Err(Error::new(ErrorKind::Other, "found key")),
            Err(_) => Ok(()),
        }
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let (mut client, mut listener) = open("127.0.0.1:3000".to_string()).await.unwrap();
        let (_, res) = future::join(timeout(WAIT ,listener.listen()), timeout(WAIT, async move {
            client.set(b"fuga".to_vec(), b"foo".to_vec()).await;
            client.get(b"fuga".to_vec()).await.unwrap()
        })).await;
        assert_eq!(res.unwrap(), b"foo".to_vec());
    }
}
