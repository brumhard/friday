use friday::http::{write, Server};
use std::{net::TcpStream, thread, time::Duration};

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let mut server: Server<TcpStream> = Server::new();

    // server.register_handler(String::from("test"));
    server.register_func("/sleep", |_r, rw| {
        thread::sleep(Duration::from_secs(5));
        write(rw, 200, "done".to_string())
    });
    server.register_func("/", |_r, rw| write(rw, 200, "whoop".to_string()));

    server.listen_and_serve("0.0.0.0:7878").unwrap();
}
