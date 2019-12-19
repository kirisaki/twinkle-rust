pub mod client;
pub mod listener;
pub mod request;
pub mod packet;
pub mod types;
pub mod errors;

use futures::future;
use tokio::time::timeout;
use std::time::{Duration};
use std::io::{Error, ErrorKind};


#[cfg(test)]
mod tests {
    use crate::*;
    use crate::client::Client;
    const WAIT: Duration = std::time::Duration::from_secs(1);

    #[tokio::test]
    async fn test_ping() -> Result<(), std::io::Error> {
        let (mut client, mut listener) = Client::open("127.0.0.1:3000").await?;
        let (_ ,res) = future::join(timeout(WAIT ,listener.listen()), client.ping()).await;
        res
    }

    #[tokio::test]
    async fn test_get_not_found() -> Result<(), std::io::Error> {
        let (mut client, mut listener) = Client::open("127.0.0.1:3000").await?;
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
        let (mut client, mut listener) = Client::open("127.0.0.1:3000").await.unwrap();
        let (_, res) = future::join(timeout(WAIT ,listener.listen()), timeout(WAIT, async move {
            client.set(b"fuga".to_vec(), b"foo".to_vec()).await;
            client.get(b"fuga".to_vec()).await.unwrap()
        })).await;
        assert_eq!(res.unwrap(), b"foo".to_vec());
    }
}
