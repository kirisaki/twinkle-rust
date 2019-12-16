# Twinkle

Binding of [Twinkle](https://github.com/kirisaki/twinkle) for Rust.

## Usage

Cargo.toml

```toml
[dependencies]
twinkle = "0.1"
futures = "0.3"
tokio = { version = "0.2", features = ["full"] }
```

main.rs

```rust
use futures::future::{join};

#[tokio::main]
async fn main(){
    let {mut client, mut listener) = twinkle::open("i27.0.0.1:9000").await.unwrap();
    use futures::future::{join};

#[tokio::main]
async fn main(){
    let (client, mut listener) = twinkle::open("127.0.0.1:9000").await.unwrap();
    join(listener.listen(), your_app(client));
}

async fn your_app(mut c: twinkle::Client) {
    c.ping().await;
    c.set(b"hoge".to_vec(), b"foo".to_vec()).await;
    c.get(b"hoge".to_vec()).await;
    c.unset(b"hoge".to_vec()).await;
}

```

## License

[BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause)
