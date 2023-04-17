use std::sync::{Arc, RwLock};

use friday::{
    asynchttp::{Router, Server},
    http::{Request, Response},
    DefaultManager, FileBackedRepo, Manager,
};
use serde::{Deserialize, Serialize};

async fn list(_: Request, manager: impl Manager + Sync + Send) -> Response {
    let list = manager.list(None).unwrap();
    Response::json(200, &list)
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ErrorBody {
    fn new(error: &str) -> ErrorBody {
        ErrorBody {
            error: error.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExampleBody {
    test: String,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("INFO"));

    let repo = FileBackedRepo::new("./testing").unwrap();
    let manager = Arc::new(RwLock::new(DefaultManager::new(repo)));

    let mut router = Router::new();
    router.post("/lol", move |r: Request| async move {
        let t: Option<ExampleBody> = r.json();
        if t.is_none() {
            return Response::json(400, &ErrorBody::new("valid json body is required"));
        }
        Response::json(200, &t.unwrap())
    });
    // router.path("/lol", move |r, JSON(t): JSON<test>| async move {
    //     Response::new(200, &"lol".to_string())
    // });
    router.path("/func", move |r| {
        let manager = Arc::clone(&manager);
        async move { list(r, manager).await }
    });
    // FIXME: the following does not work because manager is moved
    router.path("/test", move |r| {
        let manager = Arc::clone(&manager);
        async move { list(r, manager).await }
    });
    router.get("/get", move |_| async move {
        Response::json(200, &"get".to_string())
    });

    let server = Server::new(router);

    server.listen_and_serve("0.0.0.0:7878").await.unwrap()
}
