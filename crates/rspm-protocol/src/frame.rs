//! Length-prefixed JSON frame codec.

use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use rspm_core::{Result, RspmError};

const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// Writes a serializable payload as a length-prefixed JSON frame.
///
/// ```
/// # async fn write_example() -> rspm_core::Result<()> {
/// let mut buf = Vec::new();
/// rspm_protocol::frame::write_frame(
///     &mut buf,
///     &rspm_protocol::Request::Ping,
/// )
/// .await?;
/// # Ok(())
/// # }
/// ```
pub async fn write_frame<W, T>(writer: &mut W, value: &T) -> Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let payload = serde_json::to_vec(value)?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(RspmError::Protocol(format!(
            "frame exceeds {} bytes",
            MAX_FRAME_BYTES
        )));
    }

    let len = u32::try_from(payload.len())
        .map_err(|_| RspmError::Protocol("frame too large".to_owned()))?;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&payload).await?;
    Ok(())
}

/// Reads a length-prefixed JSON frame.
///
/// ```
/// # async fn read_example() -> rspm_core::Result<()> {
/// let mut buf = Vec::new();
/// rspm_protocol::frame::write_frame(&mut buf, &rspm_protocol::Request::Ping).await?;
/// let request: rspm_protocol::Request = rspm_protocol::frame::read_frame(&mut &buf[..]).await?;
/// assert!(matches!(request, rspm_protocol::Request::Ping));
/// # Ok(())
/// # }
/// ```
pub async fn read_frame<R, T>(reader: &mut R) -> Result<T>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let mut len_buf = [0_u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_FRAME_BYTES {
        return Err(RspmError::Protocol(format!(
            "frame exceeds {} bytes",
            MAX_FRAME_BYTES
        )));
    }

    let mut payload = vec![0_u8; len];
    reader.read_exact(&mut payload).await?;
    Ok(serde_json::from_slice(&payload)?)
}
