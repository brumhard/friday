use std::{collections::HashMap, future::Future, sync::Arc};

use async_trait::async_trait;


use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, ToSocketAddrs},
};

pub use crate::http_types::*;
use crate::Result;
use crate::{
    http::{Method, Request},
    Error,
};

pub struct Router {
    handlers: Vec<Route>,
    middlewares: Vec<Box<dyn Middleware + Send + Sync>>,
}

struct Route {
    path: String,
    handler: Box<dyn Handler + Send + Sync>,
    method: Option<Method>,
}

macro_rules! route_method {
    ($name:ident) => {
        route_method!($name, Some(stringify!($name).parse().unwrap()));
    };
    ($name:ident, $method:expr) => {
        pub fn $name<H: Handler + Send + Sync + 'static>(&mut self, path: &str, handler: H) {
            self.handlers.push(Route {
                path: path.to_owned(),
                handler: Box::new(handler),
                method: $method,
            })
        }
    };
}

impl Router {
    pub fn new() -> Router {
        Router {
            handlers: vec![],
            middlewares: vec![],
        }
    }
    pub fn mw<M: Middleware + Sync + Send + 'static>(&mut self, mw: M) {
        self.middlewares.push(Box::new(mw))
    }
    route_method!(path, None);
    route_method!(get);
    route_method!(post);
    route_method!(put);
    route_method!(patch);
    route_method!(delete);
}

#[async_trait]
impl Handler for Router {
    async fn handle(&self, r: Request) -> Response {
        for route in &self.handlers {
            if !r.path.starts_with(&route.path) {
                continue;
            }
            if route.method.is_some() && route.method.as_ref().unwrap() != &r.method {
                continue;
            }
            for mw in self.middlewares.iter().rev() {
                if let Some(resp) = mw.execute(r.clone()).await {
                    return resp;
                }
            }
            return route.handler.handle(r).await;
        }
        Response::json(404, &HashMap::from([("error", "not found")]))
    }
}

pub struct Server<H: Handler> {
    handler: Arc<H>,
}

impl<H: Handler + Send + Sync + 'static> Server<H> {
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
                    let resp = handler.handle(r).await;
                    write(stream, resp.status, &resp.body.unwrap()).await
                }
            });
        }
    }
}

#[async_trait]
pub trait Handler {
    async fn handle(&self, r: Request) -> Response;
}

#[async_trait]
impl<F, Fut> Handler for F
where
    Fut: Future<Output = Response> + Send + Sync,
    F: Fn(Request) -> Fut + Send + Sync,
{
    async fn handle(&self, r: Request) -> Response {
        self(r).await
    }
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
        raw_body: body,
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

#[async_trait]
pub trait Middleware {
    async fn execute(&self, r: Request) -> Option<Response>;
}

#[async_trait]
impl<F, Fut> Middleware for F
where
    Fut: Future<Output = Option<Response>> + Send + Sync,
    F: Fn(Request) -> Fut + Send + Sync,
{
    async fn execute(&self, r: Request) -> Option<Response> {
        self(r).await
    }
}
