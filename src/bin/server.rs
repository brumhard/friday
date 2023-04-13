use std::{sync::Arc};

use async_trait::async_trait;
use friday::{
    asynchttp::{write, Handler, Router, Server},
    http::Request,
    DefaultManager, FileBackedRepo, Manager,
};

use tokio::{io::AsyncWriteExt, net::TcpStream};

struct SomeHandler {
    resp: String,
}

#[async_trait]
impl Handler<TcpStream> for SomeHandler {
    async fn handle(&self, _: friday::http::Request, rw: TcpStream) {
        // sleep(Duration::from_secs(5)).await;
        write(rw, 200, &self.resp).await;
    }
}

struct Api {
    router: Router<TcpStream>,
}

impl Api {
    fn new(mngr: impl Manager + Sync + Send + 'static) -> Api {
        let mut router = Router::new();

        router.register_handler(
            "/lol",
            SomeHandler {
                resp: "lol".to_string(),
            },
        );
        router.register_handler(
            "/done",
            SomeHandler {
                resp: "done".to_string(),
            },
        );

        let mngr = ManagerWrapper::new(mngr);
        router.register_handler("/func", move |r, rw| list(r, rw, mngr.clone()));
        Api { router }
    }
}

struct ManagerWrapper<T: Manager> {
    inner: Arc<T>,
}

impl<T: Manager> ManagerWrapper<T> {
    fn new(mngr: T) -> Self {
        Self {
            inner: Arc::new(mngr),
        }
    }
}

impl<T: Manager> Clone for ManagerWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Manager> Manager for ManagerWrapper<T> {
    fn add(&self, task: &str, section: Option<&str>) -> friday::Result<()> {
        self.inner.add(task, section)
    }

    fn list(&self, section: Option<&str>) -> friday::Result<Vec<String>> {
        self.inner.list(section)
    }

    fn sections(&self) -> friday::Result<std::collections::HashMap<friday::Section, Vec<String>>> {
        self.inner.sections()
    }

    fn rm(&self, pattern: &str, section: Option<&str>) -> friday::Result<()> {
        self.inner.rm(pattern, section)
    }
}

async fn list(_: Request, rw: impl AsyncWriteExt + Unpin, manager: impl Manager + Sync + Send) {
    let list = manager.list(None).unwrap();
    write(rw, 200, &format!("{:?}", list)).await
}

#[async_trait]
impl Handler for Api {
    async fn handle(&self, r: friday::http::Request, rw: TcpStream) {
        self.router.handle(r, rw).await
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let repo = FileBackedRepo::new("./testing").unwrap();
    let manager = DefaultManager::new(repo);

    let api = Api::new(manager);
    let server = Server::new(api);

    server.listen_and_serve("0.0.0.0:7878").await.unwrap()
}
