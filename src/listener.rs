use std::io::{Error, ErrorKind};
use std::sync::Arc;

use uuid::Uuid;

use tokio::sync::Mutex;
use tokio::net::udp::RecvHalf;

use crate::types::*;
use crate::errors::*;

pub struct Listener {
    pub sock: RecvHalf,
    pub tabl: Arc<Mutex<Table>>,
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
