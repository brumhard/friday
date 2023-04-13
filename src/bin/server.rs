use async_trait::async_trait;
use friday::asynchttp::{write, Handler, Server};
use tokio::{net::TcpStream};

struct SomeHandler;

#[async_trait]
impl Handler<TcpStream> for SomeHandler {
    async fn handle(&self, _: friday::http::Request, rw: TcpStream) {
        write(rw, 200, "done").await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let handler = SomeHandler {};
    let server = Server::new(handler);

    server.listen_and_serve("0.0.0.0:7878").await.unwrap()
}
