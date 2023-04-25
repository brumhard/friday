#![warn(clippy::pedantic)]

use anyhow::Result;
use friday_core::{DefaultManager, Manager, Repo};
use http::StatusCode;
use indexmap::IndexMap;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    key_value::Store,
};

struct SpinRepo {
    store: Store,
}

impl SpinRepo {
    fn new() -> Result<Self> {
        let store = Store::open_default()?;
        Ok(SpinRepo { store })
    }
}

impl Repo for SpinRepo {
    // TODO: replace unwrap with propper error handling
    fn create(
        &self,
        task: &str,
        section: friday_core::Section,
    ) -> std::result::Result<(), friday_core::Error> {
        let mut tasks = self.list(section.clone())?;
        tasks.push(task.to_string());
        self.store
            .set(section.to_string(), serde_json::to_string(&tasks).unwrap())
            .unwrap();
        Ok(())
    }

    fn list(
        &self,
        section: friday_core::Section,
    ) -> std::result::Result<Vec<String>, friday_core::Error> {
        let tasks_json = self.store.get(section.to_string()).unwrap_or_default();
        let tasks: Vec<String> = serde_json::from_slice(&tasks_json).unwrap_or_default();
        Ok(tasks)
    }

    fn list_all(
        &self,
    ) -> std::result::Result<IndexMap<friday_core::Section, Vec<String>>, friday_core::Error> {
        let test = self.store.get_keys().unwrap();
        Ok(test
            .iter()
            .map(|t| {
                (
                    t.as_str().parse().unwrap(),
                    self.list(t.parse().unwrap()).unwrap(),
                )
            })
            .collect())
    }

    fn delete(
        &self,
        _task: &str,
        _section: friday_core::Section,
    ) -> std::result::Result<(), friday_core::Error> {
        todo!()
    }
}

/// A simple Spin HTTP component.
#[allow(clippy::needless_pass_by_value)] // that's spins interface
#[http_component]
fn handle_friday(req: Request) -> Result<Response> {
    let manager = DefaultManager::new(SpinRepo::new()?);

    if req.method() == http::Method::POST {
        let body = req.body().as_deref().unwrap_or(&[]);
        if body.is_empty() {
            return Ok(http::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Some("body is required".into()))?);
        }
        let task: String = serde_json::from_slice(body)?;
        manager.add(&task, None)?;
    }
    let test = manager.sections()?;
    let test_str = serde_json::to_string(&test)?;

    Ok(http::Response::builder()
        .status(200)
        .body(Some(test_str.into()))?)
}
