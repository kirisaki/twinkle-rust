use std::net::{UdpSocket};
use std::convert::TryFrom;

// limitation of UDP
const BUF_SIZE: usize = 64 * 1024;

pub struct Client {
    socket: UdpSocket,
}

pub async fn open(tx: String) -> Result<Client, std::io::Error> {
    let s = UdpSocket::bind("127.0.0.1:3000")?;
    s.connect(tx);
    Ok (Client {
        socket: s
    })
}

impl Client {
    pub async fn ping(&self) -> Result<(), std::io::Error> {
        let mut req: Vec<u8> = vec![0x00];

        let _ = self.socket.send(req.as_mut_slice())?;

        let mut buf = [0; BUF_SIZE];
        let amt = self.socket.recv(&mut buf)?;
        return Err(std::io::Error::new(std::io::ErrorKind::Other, buf[1].to_string()));

        match parse_body(&buf, amt) {
            Some((_, _, _)) => Ok(()),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "failed parsing")),
        }
    }

    pub async fn get(&self, mut key: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
        let keylen: i16 = TryFrom::try_from(key.len()).unwrap();
        let h: u8 = TryFrom::try_from(keylen/256).unwrap();
        let l: u8 = TryFrom::try_from(keylen).unwrap();
        let mut req: Vec<u8> = vec![];
        let mut header: Vec<u8> = vec![0x01, h, l];
        req.append(&mut header);
        req.append(&mut key);
        let _ = self.socket.send(req.as_mut_slice())?;

        let mut buf = [0; BUF_SIZE];
        let amt = self.socket.recv(&mut buf)?;

        match parse_body(&buf, amt) {
            Some((_, _, value)) => Ok(value.to_vec()),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "failed parsing")),
        }
    }
}

fn parse_body<'a>(buf: &'a [u8; BUF_SIZE], amt: usize) -> Option<(u8, &'a [u8], &'a [u8])> {
    if amt >= 3 {
        let cmd = buf[0];
        let high: usize = From::from(buf[1]);
        let low: usize = From::from(buf[2]);
        let keylen = high * 256 + low;
        let key = if keylen == 0 {
            &[]
        } else {
            &buf[3..3+keylen]
        };
        let value = if 3 + keylen == amt {
            &[]
        } else {
            &buf[3+keylen..amt]
        };
        return Some((cmd, key, value));
    } else if amt == 1 {
        return Some((buf[0], &[], &[]));
    } else {
        return None;
    }
}


#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn it_works() {
        let c = open("127.0.0.1:3000".to_string()).await.unwrap();
        assert!(match c.ping().await {
            Ok(_) => true,
            Err(e) => {
                println!("error: {:?}", e);
                false
            },
        })
    }
}
