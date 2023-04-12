use friday::http::{write, Server};
use std::net::TcpStream;

fn main() {
    let mut server: Server<TcpStream> = Server::new();

    // server.register_handler(String::from("test"));
    server.register_func("/".to_owned(), |_r, rw| write(rw, 200, "whoop".to_string()));

    server.listen_and_serve("0.0.0.0:7878").unwrap();
}
