use crate::{Repo, Result};
use std::str;

trait Manager {
    fn add(&self, task: &str, section: Option<&str>) -> Result<()>;
    fn list(&self, section: Option<&str>) -> Result<Vec<String>>;
    fn rm(&self, pattern: &str, section: Option<&str>) -> Result<()>;
    fn manage(&self) -> Result<()> {
        // TODO: this is probably not the right pattern for this
        panic!("this should not be used if not implemented")
    }
}

struct DefaultManager<T: Repo> {
    repo: T,
}

impl<T: Repo> DefaultManager<T> {
    fn new(repo: T) -> DefaultManager<T> {
        DefaultManager { repo }
    }
}
