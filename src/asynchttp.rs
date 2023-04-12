use tokio::io::AsyncWriteExt;

pub async fn write(mut writer: impl AsyncWriteExt + Unpin, status: u16, body: &str) {
    let content_length = body.len();
    let reason = match status {
        500 => "Internal Server Error",
        501 => "Not Implemented",
        400 => "Bad Request",
        _ => "OK",
    };

    let response = format!(
        "\
HTTP/1.1 {status} {reason}
Content-Length: {content_length}

{body}"
    );

    writer
        .write_all(response.as_bytes())
        .await
        .unwrap_or_else(|e| log::error!("failed to write response: {e}"))
}
