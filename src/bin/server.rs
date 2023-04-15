use std::sync::{Arc, RwLock};

use friday::{
    asynchttp::{write, Router, Server},
    http::Request,
    DefaultManager, FileBackedRepo, Manager,
};

use tokio::io::AsyncWriteExt;

async fn list(_: Request, rw: impl AsyncWriteExt + Unpin, manager: impl Manager + Sync + Send) {
    let list = manager.list(None).unwrap();
    write(rw, 200, &format!("{:?}", list)).await
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let repo = FileBackedRepo::new("./testing").unwrap();
    let manager = Arc::new(RwLock::new(DefaultManager::new(repo)));

    let mut router = Router::new();
    router.register_handler("/lol", move |_, rw| write(rw, 200, "lol"));
    router.register_handler("/done", move |_, rw| write(rw, 200, "done"));
    router.register_handler("/func", move |r, rw| {
        let manager = Arc::clone(&manager);
        async move { list(r, rw, manager).await }
    });

    let server = Server::new(router);

    server.listen_and_serve("0.0.0.0:7878").await.unwrap()
}
