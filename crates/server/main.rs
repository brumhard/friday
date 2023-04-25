use aide::{
    axum::{
        routing::{get, post},
        ApiRouter,
    },
    openapi::{Info, OpenApi},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use friday_core::{DefaultManager, FileBacked, Manager, Section};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

type Mngr = Arc<RwLock<dyn Manager + Sync + Send>>;

#[tokio::main]
pub async fn main() {
    let repo = FileBacked::new("./testing").unwrap();
    let manager = Arc::new(RwLock::new(DefaultManager::new(repo)));
    let routes = ApiRouter::new()
        .api_route("/tasks", get(handle_get_tasks))
        .api_route("/tasks/:section", get(handle_get_tasks_in_section))
        // https://github.com/tamasfe/aide/pull/38
        // .api_route(
        //     "/tasks",
        //     post(|| async { Redirect::permanent("/tasks/dump") }),
        // )
        .api_route("/tasks/:section", post(handle_post_tasks))
        .route(
            "/api.json",
            get(|Extension(api): Extension<OpenApi>| async { Json(api) }),
        )
        .with_state(manager);

    let mut api = OpenApi {
        info: Info {
            description: Some("Friday API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(
            routes
                .finish_api(&mut api)
                .layer(Extension(api))
                .into_make_service(),
        )
        .await
        .unwrap();
}

#[allow(clippy::unused_async)] // required for handler function signature
async fn handle_get_tasks(State(mngr): State<Mngr>) -> Result<HashMap<Section, Vec<String>>> {
    let sections = mngr.read().unwrap().sections().map_err(to_http_err)?;
    Ok((StatusCode::OK, Json(sections)))
}

#[allow(clippy::unused_async)] // required for handler function signature
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

#[allow(clippy::unused_async)] // required for handler function signature
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

#[derive(Serialize, Deserialize, JsonSchema)]
struct CreateTask {
    task: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ListResponse<T> {
    items: Vec<T>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ErrResponse {
    message: String,
}

#[allow(clippy::needless_pass_by_value)] // easier to use with this signature
fn to_http_err(e: friday_core::Error) -> (StatusCode, Json<ErrResponse>) {
    let status = match &e {
        e if e.to_string().contains("not found") => StatusCode::NOT_FOUND,
        friday_core::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ErrResponse {
            message: e.to_string(),
        }),
    )
}
