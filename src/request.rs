use uuid::Uuid;

use crate::packet::*;
use crate::types::*;
use crate::errors::*;

#[derive(Debug, PartialEq)]
pub enum Request {
    Ping,
    Get(Bytes),
    Set(Bytes, Bytes),
    Unset(Bytes),
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
