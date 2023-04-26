#![warn(clippy::pedantic)]

use aide::{
    axum::{
        routing::{get, post},
        ApiRouter,
    },
    openapi::{Info, OpenApi},
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use friday_core::{DefaultManager, FileBacked, Manager, Section};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::prelude::*;

type Mngr = Arc<RwLock<dyn Manager + Sync + Send>>;

#[tokio::main]
pub async fn main() {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("tower_http::trace::on_response", Level::DEBUG)
        .with_target("tower_http::trace::on_request", Level::DEBUG)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_default(Level::INFO);
    let tracing_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();

    let repo = FileBacked::new("./testing").unwrap();
    let manager = Arc::new(RwLock::new(DefaultManager::new(repo)));
    let routes = ApiRouter::new()
        .api_route("/tasks", get(handle_get_tasks))
        .api_route("/tasks/:section", get(handle_get_tasks_in_section))
        .api_route("/tasks/:section", post(handle_post_tasks))
        .api_route(
            "/tasks",
            // NOTE: wait for this https://github.com/tamasfe/aide/pull/38
            post(|| async {
                Response::builder()
                    .status(StatusCode::PERMANENT_REDIRECT)
                    .header(header::LOCATION, "/tasks/dump")
                    .body(Body::empty())
                    .unwrap()
            }),
        )
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

    tracing::info!("serving on port 3000");
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(
            routes
                .finish_api(&mut api)
                .layer(Extension(api))
                .layer(TraceLayer::new_for_http())
                .into_make_service(),
        )
        .await
        .unwrap();
}

#[allow(clippy::unused_async)] // required for handler function signature
async fn handle_get_tasks(State(mngr): State<Mngr>) -> Result<IndexMap<Section, Vec<String>>> {
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
) -> Result<IndexMap<Section, Vec<String>>> {
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
