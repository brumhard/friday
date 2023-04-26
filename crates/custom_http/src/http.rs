use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    str::{self},
    thread,
};

use rayon::prelude::*;

pub use crate::http_types::*;
use crate::{Error, Result};

struct FuncWrapper<'func, T: Write> {
    f: Box<dyn Fn(Request, T) + Send + Sync + 'func>,
}

impl<'func, T: Write> FuncWrapper<'func, T> {
    fn new<F: Fn(Request, T) + Send + Sync + 'func>(f: F) -> FuncWrapper<'func, T> {
        FuncWrapper { f: Box::new(f) }
    }
}

impl<T: Write> Handler<T> for FuncWrapper<'_, T> {
    fn handle(&self, r: Request, rw: T) {
        let f = &self.f;
        f(r, rw);
    }
}

pub struct Server<'handler, T: Write> {
    handlers: Vec<(String, Box<dyn Handler<T> + Send + Sync + 'handler>)>,
}

impl Server<'_, TcpStream> {
    pub fn listen_and_serve<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        // 1. implementation with custom threadpool
        // this was using the threadpool from https://doc.rust-lang.org/stable/book/ch20-02-multithreaded.html
        // before, but the thread::spawn only supports static contexts which makes it
        // pretty hard to use it with self.
        // see https://users.rust-lang.org/t/how-to-use-self-while-spawning-a-thread-from-method/8282.
        //
        // 2. implementation with rayon
        // using rayon par_bridge will put all streams on separate threads on it's own.
        // Ofc if all threads are blocked this doesn't allow more connections.

        //
        // 3. implementation with tokio
        // To not block any threads on io tokio is a nice replacement: https://tokio.rs/tokio/tutorial.
        // This is not implemented here but in the asynchttp module.
        listener.incoming().par_bridge().for_each(|stream| match stream {
            Ok(stream) => {
                log::debug!("Thread ID: {:?} - Handling request", thread::current().id(),);
                self.handle_connection(stream);
            }
            Err(e) => log::error!("error in TCP connection: {e}"),
        });

        Ok(())
    }

    fn handle_connection(&self, stream: TcpStream) {
        let r = match parse_request(&stream) {
            Ok(r) => r,
            Err(e) => {
                write(stream, 400, &format!("invalid request: {e}"));
                log::error!("invalid request: {e}");
                return;
            }
        };
        for (path, handler) in &self.handlers {
            if r.path.contains(path) {
                handler.handle(r, stream);
                break;
            }
        }
    }
}

impl<'handler, T: Write + 'handler> Server<'handler, T> {
    pub fn new() -> Server<'handler, T> {
        Server { handlers: Vec::new() }
    }

    pub fn register_handler<H: Handler<T> + Send + Sync + 'handler>(
        &mut self,
        path: &str,
        handler: H,
    ) {
        self.handlers.push((path.to_owned(), Box::new(handler)));
    }

    pub fn register_func<F>(&mut self, path: &str, f: F)
    where
        F: Fn(Request, T) + Send + Sync + 'handler,
    {
        self.register_handler(path, FuncWrapper::new(f));
    }
}

impl<'handler, T: Write + 'handler> Default for Server<'handler, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Handler<T: Write> {
    fn handle(&self, r: Request, rw: T);
}

pub fn write(mut writer: impl Write, status: u16, body: &str) {
    let content_length = body.len();
    let reason = match status {
        500 => "Internal Server Error",
        501 => "Not Implemented",
        400 => "Bad Request",
        _ => "OK",
    };

    write!(
        writer,
        "\
HTTP/1.1 {status} {reason}
Content-Length: {content_length}

{body}"
    )
    .unwrap_or_else(|e| log::error!("failed to write response: {e}"));
}

fn parse_request(reader: impl Read) -> Result<Request> {
    let mut reader = BufReader::new(reader);
    let mut lines = reader.by_ref().lines();
    let first_line = match lines.next() {
        Some(Ok(line)) => line,
        // error for empty will be handled in parse_first_line
        None => String::new(),
        Some(Err(e)) => {
            return Err(e.into());
        }
    };
    let (method, path, _) = parse_first_line(&first_line)?;

    let mut headers: HashMap<String, String> = HashMap::new();
    for line_result in lines {
        let line = line_result?;
        if line.is_empty() {
            break;
        }
        let (key, value) = line.split_once(':').unwrap_or_default();
        headers.insert(key.trim().to_string(), value.trim().to_string());
    }

    let body = read_body(reader, &headers)?;

    Ok(Request { method, path, headers, raw_body: body })
}

fn parse_first_line(s: &str) -> Result<(Method, String, String)> {
    let parts: Vec<&str> = s.splitn(3, ' ').collect();
    if parts.len() != 3 {
        return Err(Error::InvalidArgument(
            "invalid number of elements in first HTTP line".to_string(),
        ));
    }
    Ok((parts[0].parse()?, parts[1].to_string(), parts[2].to_string()))
}

fn read_body(mut reader: impl Read, headers: &HashMap<String, String>) -> Result<Option<String>> {
    let mut body: Option<String> = None;
    let mut body_buffer;
    // https://www.rfc-editor.org/rfc/rfc7230#section-3.2
    if headers.get("Transfer-Encoding").is_some() {
        return Err(Error::InvalidArgument("Tranfer-Encoding is not supported".to_string()));
    }
    if let Some(size_string) = headers.get("Content-Length") {
        let size: usize = size_string.parse().unwrap_or_default();
        body_buffer = vec![0; size];
        reader.read_exact(&mut body_buffer)?;
        body = Some(String::from_utf8_lossy(&body_buffer).into());
    }
    Ok(body)
}
