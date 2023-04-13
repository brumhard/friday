use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

pub use crate::http_types::*;
use crate::Result;
use crate::{
    http::{Method, Request},
    Error,
};

// struct FuncWrapper<T: AsyncWriteExt + Unpin> {
//     f: Box<dyn Async(Request, T) + Send + Sync>,
// }

// impl<T: AsyncWriteExt + Unpin> FuncWrapper<T> {
//     fn new<F: Fn(Request, T) + Send + Sync>(f: F) -> FuncWrapper<T> {
//         FuncWrapper { f: Box::new(f) }
//     }
// }

// #[async_trait]
// impl<T: AsyncWriteExt + Unpin> Handler<T> for FuncWrapper<T> {
//     async fn handle(&self, r: Request, rw: T) {
//         let f = &self.f;
//         f(r, rw)
//     }
// }

pub struct Router<T: AsyncWriteExt + Unpin> {
    handlers: Vec<(String, Box<dyn Handler<T> + Send + Sync>)>,
}

impl<T: AsyncWriteExt + Unpin> Router<T> {
    pub fn new() -> Router<T> {
        Router { handlers: vec![] }
    }
    pub fn register_handler<H: Handler<T> + Send + Sync + 'static>(
        &mut self,
        path: &str,
        handler: H,
    ) {
        self.handlers.push((path.to_owned(), Box::new(handler)))
    }

    // pub fn register_func<F>(&mut self, path: &str, f: F)
    // where
    //     F: Fn(Request, T) + Send + Sync + 'static,
    // {
    //     self.register_handler(path, FuncWrapper::new(f))
    // }
}

#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send> Handler<T> for Router<T> {
    async fn handle(&self, r: Request, rw: T) {
        for (path, handler) in &self.handlers {
            if r.path.contains(path) {
                handler.handle(r, rw).await;
                return;
            }
        }
        write(rw, 404, "not found").await;
    }
}

pub struct Server<H: Handler<TcpStream> + Send + Sync> {
    handler: Arc<H>,
}

impl<H: Handler<TcpStream> + Send + Sync + 'static> Server<H> {
    pub fn new(handler: H) -> Server<H> {
        Server {
            handler: Arc::new(handler),
        }
    }

    pub async fn listen_and_serve<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        log::info!("running server");
        let listener = TcpListener::bind(addr).await?;

        loop {
            // TODO: add select here to wait for cancellation
            let (mut stream, ip) = listener.accept().await?;
            log::debug!("got request from {ip}");

            tokio::spawn({
                let handler = Arc::clone(&self.handler);
                async move {
                    let r = match parse_request(&mut stream).await {
                        Ok(r) => r,
                        Err(e) => {
                            write(stream, 400, &format!("invalid request: {e}")).await;
                            log::error!("invalid request: {e}");
                            return;
                        }
                    };
                    log::debug!("handling request");
                    handler.handle(r, stream).await
                }
            });
        }
    }
}

#[async_trait]
pub trait Handler<T: AsyncWriteExt + Unpin> {
    async fn handle(&self, r: Request, rw: T);
}

pub async fn write(mut writer: impl AsyncWriteExt + Unpin, status: u16, body: &str) {
    let content_length = body.len();
    let reason = match status {
        500 => "Internal Server Error",
        501 => "Not Implemented",
        400 => "Bad Request",
        404 => "Not Found",
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

async fn parse_request<R: AsyncReadExt + Unpin>(reader: R) -> Result<Request> {
    let mut reader = BufReader::new(reader);
    let mut line_buf = String::new();

    reader.read_line(&mut line_buf).await?;
    let (method, path, _) = parse_first_line(&line_buf)?;
    log::debug!("got first line: {method} {path}");

    let mut headers = HashMap::new();
    loop {
        line_buf.clear();
        reader.read_line(&mut line_buf).await?;
        let line = line_buf.trim();
        if line.is_empty() {
            break;
        }
        let (key, value) = line.split_once(':').unwrap_or_default();
        headers.insert(key.trim().to_string(), value.trim().to_string());
    }
    log::debug!("got headers: {:?}", headers);

    let body = read_body(&mut reader, &headers).await?;
    log::debug!("got body: {:?}", body);

    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

fn parse_first_line(s: &str) -> Result<(Method, String, String)> {
    let parts: Vec<&str> = s.splitn(3, ' ').collect();
    if parts.len() != 3 {
        return Err(Error::InvalidArgument(
            "invalid number of elements in first HTTP line".to_string(),
        ));
    }
    Ok((
        parts[0].parse()?,
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

async fn read_body(
    mut reader: impl AsyncReadExt + Unpin,
    headers: &HashMap<String, String>,
) -> Result<Option<String>> {
    let mut body: Option<String> = None;
    let mut body_buffer;
    // https://www.rfc-editor.org/rfc/rfc7230#section-3.2
    if headers.get("Transfer-Encoding").is_some() {
        return Err(Error::InvalidArgument(
            "Tranfer-Encoding is not supported".to_string(),
        ));
    }
    if let Some(size_string) = headers.get("Content-Length") {
        let size: usize = size_string.parse().unwrap_or_default();
        body_buffer = vec![0; size];
        reader.read_exact(&mut body_buffer).await?;
        body = Some(String::from_utf8_lossy(&body_buffer).into());
    }
    Ok(body)
}
