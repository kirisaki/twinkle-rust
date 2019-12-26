//! Client library for [twinkle](https://github.com/kirisaki/twinkle).
//!
//! Cargo.toml
//!
//! ```toml
//! [dependencies]
//! twinkle = "0.1"
//! futures = "0.3"
//! tokio = { version = "0.2", features = ["full"] }
//! ```
//! 
//! main.rs
//! 
//! ```rust
//! use futures::future::{join};
//! use twinkle::client::Client;
//! 
//! #[tokio::main]
//! async fn main(){
//!     let (client, manager) = Client::open("127.0.0.1:3000").await.unwrap();
//!     join(manager.run(), your_app(client));
//! }
//! 
//! async fn your_app(mut c: Client) {
//!     c.ping().await;
//!     c.set(b"hoge".to_vec(), b"foo".to_vec()).await;
//!     c.get(b"hoge".to_vec()).await;
//!     c.unset(b"hoge".to_vec()).await;
//! }
//! 
//! ```

pub mod client;
mod listener;
pub mod request;
mod packet;
mod dispatcher;
mod types;
mod errors;

use futures::future::join;
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
        let (mut client, manager) = Client::open("127.0.0.1:3000").await?;
        let (_, res) = join(
            timeout(WAIT, manager.run()),
            client.ping()).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    #[tokio::test]
    async fn test_get_not_found() -> Result<(), std::io::Error> {
        let (mut client, manager) = Client::open("127.0.0.1:3000").await?;
        let (_, res) = join(
            timeout(WAIT, manager.run()),
            timeout(WAIT, async move {
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
        let (mut client, manager) = Client::open("127.0.0.1:3000").await.unwrap();
        let (_, res) = join(
            timeout(WAIT, manager.run()),
            timeout(WAIT, async move {
                client.set(b"fuga".to_vec(), b"foo".to_vec()).await;
                client.get(b"fuga".to_vec()).await.unwrap()
        })).await;
        assert_eq!(res.unwrap(), b"foo".to_vec());
    }
}
