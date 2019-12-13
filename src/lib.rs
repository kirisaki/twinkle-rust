use tokio::net::UdpSocket;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};
use futures::future;
use tokio::sync::Mutex;
use std::sync::{Arc};


// limitation of UDP
const BUF_SIZE: usize = 64 * 1024;
const UUID_LEN: usize = 16;

type Bytes = Vec<u8>;
type Table = HashMap<Uuid, Sender<Bytes>>;

pub struct Client {
    sock: UdpSocket,
    tabl: Table,
}

#[derive(Debug, PartialEq)]
enum Request {
    Ping,
    Get(Bytes),
    Set(Bytes, Bytes),
    Unset(Bytes),
}

pub async fn open(tx: String) -> Result<Client, std::io::Error> {
    let sock = UdpSocket::bind("127.0.0.1:0").await?;
    sock.connect(tx).await?;
    let tabl = HashMap::new();
    let c = Client{sock, tabl};
    Ok(c)
}


impl Client {
    pub async fn ping(&mut self) -> Result<(), std::io::Error> {
        let rx = Request::Ping.issue().dispatch(self).await?;
        println!("{:?}", rx.recv());
        Ok(())
    }

    pub async fn listen(&mut self) -> Result<(), std::io::Error> {
        let Client{sock, tabl} = self;
        let mut buf = vec![0; BUF_SIZE];
        loop {
            let amt = sock.recv(&mut buf).await?;
            let e = std::io::Error::new(std::io::ErrorKind::Other, "failed parsing");
            if amt < UUID_LEN + 1 {
                return Err(e);
            } else {
                if buf[0] == 0x02 {
                    return Err(e);
                } else {
                    let uuid = Uuid::from_slice(&buf[1..UUID_LEN]).unwrap();
                    let val = buf[UUID_LEN + 1..amt].to_vec();
                    match tabl.get(&uuid) {
                        Some(c) => {c.send(val); return Ok(())},
                        None => return Err(e),
                    }
                }
            }

        }
    }
}


impl Request {
    fn issue(self) -> (Packet) {
        let uuid = Uuid::new_v4();
        let mut cont = match self {
            Request::Ping =>
                vec![0x01],
            _ => unimplemented!(),
        };
        cont.append(&mut uuid.as_bytes().to_vec());
        Packet{cont, uuid}
    }
}

struct Packet {
    cont: Bytes,
    uuid: Uuid,
}

impl Packet {
    async fn dispatch(self,  c: &mut Client) -> Result<Receiver<Bytes>, std::io::Error> {
        let Packet{cont, uuid} = self;
        let (tx, rx) = channel();
        c.tabl.insert(uuid, tx);
        c.sock.send(&cont).await?;
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn it_works() -> Result<(), std::io::Error> {
        let mut client = open("127.0.0.1:3000".to_string()).await?;
        future::try_join(client.listen(), client.ping());
        Ok(())
    }
}
