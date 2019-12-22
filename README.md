# Twinkle

Binding of [Twinkle](https://github.com/kirisaki/twinkle) for Rust.

[![Actions Status](https://github.com/kirisaki/twinkle-rust/workflows/test-twinkle/badge.svg)](https://github.com/kirisaki/twinkle-rust/actions)
[![twinkle at crates.io](https://img.shields.io/crates/v/twinkle.svg)](https://crates.io/crates/twinkle)
[![twinkle at docs.rs](https://docs.rs/twinkle/badge.svg)](https://docs.rs/crate-name)

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
use futures::future::{join3};

#[tokio::main]
async fn main(){
    let (client, mut listener, mut dispatcher) = twinkle::open("127.0.0.1:3000").await.unwrap();
    join(listener.listen(), dispatcher.run(), your_app(client));
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
