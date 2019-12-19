use std::collections::HashMap;
use std::io::Error;

use tokio::time::Duration;
use tokio::sync::mpsc::Sender;

use uuid::Uuid;

pub type Bytes = Vec<u8>;
pub type Table = HashMap<Uuid, Sender<Result<Bytes, Error>>>;

// limitation of UDP
pub const BUF_SIZE: usize = 64 * 1024;
pub const UUID_LEN: usize = 16;
pub const TIMEOUT: Duration = Duration::from_secs(1);
