use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Redirect},
    routing::{get, post},
    Json, Router,
};
use friday::{DefaultManager, FileBackedRepo, Manager, Section};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

type Mngr = Arc<RwLock<dyn Manager + Sync + Send>>;

pub async fn main() {
    let repo = FileBackedRepo::new("./testing").unwrap();
    let manager = Arc::new(RwLock::new(DefaultManager::new(repo)));
    let routes = Router::new()
        .route("/tasks", get(handle_get_tasks))
        .route("/tasks/:section", get(handle_get_tasks_in_section))
        .route(
            "/tasks",
            post(|| async { Redirect::permanent("/tasks/dump") }),
        )
        .route("/tasks/:section", post(handle_post_tasks))
        .with_state(manager);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();
}

async fn handle_get_tasks(State(mngr): State<Mngr>) -> Result<HashMap<Section, Vec<String>>> {
    let sections = mngr.read().unwrap().sections().map_err(to_http_err)?;
    Ok((StatusCode::OK, Json(sections)))
}

async fn handle_get_tasks_in_section(
    Path(section): Path<Section>,
    State(mngr): State<Mngr>,
) -> Result<ListResponse<String>> {
    let items = mngr
        .read()
        .unwrap()
        .list(Some(&section.to_string()))
        .map_err(to_http_err)?;
    Ok((StatusCode::OK, Json(ListResponse { items })))
}

async fn handle_post_tasks(
    Path(section): Path<Section>,
    State(mngr): State<Mngr>,
    Json(input): Json<CreateTask>,
) -> Result<HashMap<Section, Vec<String>>> {
    mngr.write()
        .unwrap()
        .add(&input.task, Some(&section.to_string()))
        .map_err(to_http_err)?;
    let sections = mngr.read().unwrap().sections().map_err(to_http_err)?;
    Ok((StatusCode::OK, Json(sections)))
}

type Result<T> = std::result::Result<(StatusCode, Json<T>), (StatusCode, Json<ErrResponse>)>;

#[derive(Serialize, Deserialize)]
struct CreateTask {
    task: String,
}

#[derive(Serialize, Deserialize)]
struct ListResponse<T> {
    items: Vec<T>,
}
#[derive(Serialize, Deserialize)]
struct ErrResponse {
    message: String,
}
fn to_http_err(e: friday::Error) -> (StatusCode, Json<ErrResponse>) {
    let status = match &e {
        e if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
        friday::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ErrResponse {
            message: e.to_string(),
        }),
    )
}
