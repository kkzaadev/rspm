use std::error::Error;

use rspm_protocol::Request;
use rspm_protocol::frame::{read_frame, write_frame};

#[tokio::test]
async fn round_trips_request_frame() -> Result<(), Box<dyn Error>> {
    let mut buf = Vec::new();
    write_frame(&mut buf, &Request::Ping).await?;

    let request: Request = read_frame(&mut &buf[..]).await?;
    assert!(matches!(request, Request::Ping));
    Ok(())
}
