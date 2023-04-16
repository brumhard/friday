use std::sync::{Arc, RwLock};

use friday::{
    asynchttp::{Router, Server},
    http::{Request, Response},
    DefaultManager, FileBackedRepo, Manager,
};

async fn list(_: Request, manager: impl Manager + Sync + Send) -> Response {
    let list = manager.list(None).unwrap();
    Response::new(200, &list)
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let repo = FileBackedRepo::new("./testing").unwrap();
    let manager = Arc::new(RwLock::new(DefaultManager::new(repo)));

    let mut router = Router::new();
    router.path("/lol", move |_| async move {
        Response::new(200, &"lol".to_string())
    });
    router.path("/func", move |r| {
        let manager = Arc::clone(&manager);
        async move { list(r, manager).await }
    });
    router.get("/get", move |_| async move {
        Response::new(200, &"get".to_string())
    });

    let server = Server::new(router);

    server.listen_and_serve("0.0.0.0:7878").await.unwrap()
}
