use friday::{http::Server, Error, Result};
use std::{
    collections::HashMap,
    env,
    fmt::{self},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process,
    str::{self, FromStr},
    sync::{mpsc, Arc, Mutex},
    thread,
};

fn main() {
    let mut server: Server<TcpStream> = Server::new();

    server.register_func(String::from("test"), |r, mut rw| {
        writeln!(rw, "testing").unwrap()
    })
}
