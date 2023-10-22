use anyhow::Result;
use std::path::Path;
use tokio_uring::fs::File;

use crate::infra::result::WrapResult;

const COMMON_FILE_SIZE: usize = 1024 * 1024 * 4; // 4M
const BUFFER_SIZE: usize = 1024 * 1024 * 10; // 10M

// TODO: fix clippy warn
#[allow(clippy::future_not_send)]
pub async fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let file = File::open(path).await?;

    let mut start = 0;
    let mut buf = vec![0; BUFFER_SIZE];
    let mut bytes = Vec::with_capacity(COMMON_FILE_SIZE);
    loop {
        let (n, read) = file.read_at(buf, start).await;
        let n = n?;
        if n == 0 {
            break;
        } else {
            bytes.extend_from_slice(&read[..n]);
        }
        buf = read;
        start += n as u64;
    }

    file.close().await?;
    bytes.wrap_ok()
}
