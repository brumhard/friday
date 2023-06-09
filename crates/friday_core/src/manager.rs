use std::{
    str,
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;

use crate::{error::Result, Error, Repo, Section};

pub trait Manager {
    fn add(&self, task: &str, section: Option<&str>) -> Result<()>;
    fn list(&self, section: Option<&str>) -> Result<Vec<String>>;
    fn sections(&self) -> Result<IndexMap<Section, Vec<String>>>;
    fn rm(&self, pattern: &str, section: Option<&str>) -> Result<()>;
}

impl<T: Manager> Manager for Arc<RwLock<T>> {
    fn add(&self, task: &str, section: Option<&str>) -> Result<()> {
        self.write().unwrap().add(task, section)
    }

    fn list(&self, section: Option<&str>) -> Result<Vec<String>> {
        self.read().unwrap().list(section)
    }

    fn sections(&self) -> Result<IndexMap<Section, Vec<String>>> {
        self.read().unwrap().sections()
    }

    fn rm(&self, pattern: &str, section: Option<&str>) -> Result<()> {
        self.write().unwrap().rm(pattern, section)
    }
}

pub struct DefaultManager<T: Repo> {
    repo: T,
}

impl<T: Repo> DefaultManager<T> {
    pub fn new(repo: T) -> DefaultManager<T> {
        DefaultManager { repo }
    }
}

impl<T: Repo> Manager for DefaultManager<T> {
    fn add(&self, task: &str, section: Option<&str>) -> Result<()> {
        if task.trim().is_empty() {
            return Err(Error::InvalidArgument("expected non-empty task".to_string()));
        }

        self.repo.create(task, section.into())
    }

    fn sections(&self) -> Result<IndexMap<Section, Vec<String>>> {
        self.repo.list_all()
    }

    fn list(&self, section: Option<&str>) -> Result<Vec<String>> {
        self.repo.list(section.into())
    }

    fn rm(&self, pattern: &str, section: Option<&str>) -> Result<()> {
        let matching_tasks: Vec<String> =
            self.repo.list(section.into())?.into_iter().filter(|t| t.contains(pattern)).collect();
        if matching_tasks.len() > 1 {
            return Err(Error::InvalidArgument(format!(
                "found more than one match for pattern {pattern}"
            )));
        }
        if matching_tasks.is_empty() {
            return Err(Error::InvalidArgument(format!("no match found for pattern {pattern}")));
        }
        self.repo.delete(matching_tasks.get(0).unwrap(), section.into())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::{MockRepo, Section};

    #[test]
    fn test_rm_errors_on_multiple_matches() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_list()
            .returning(|_| Ok(vec!["some_task".to_string(), "some_other_task".to_string()]));

        let mngr = DefaultManager { repo: mock_repo };
        assert!(mngr.rm("some", Some("section")).is_err());
    }

    #[test]
    fn test_rm_errors_on_no_matches() {
        let mut mock_repo = MockRepo::new();
        mock_repo.expect_list().returning(|_| Ok(vec![]));

        let mngr = DefaultManager { repo: mock_repo };
        assert!(mngr.rm("some", Some("section")).is_err());
    }

    #[test]
    fn test_rm_works() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_list()
            .times(1)
            .returning(|_| Ok(vec!["some".to_string(), "other".to_string()]));
        mock_repo
            .expect_delete()
            .with(eq("some"), eq(Section::Dump))
            .times(1)
            .returning(|_, _| Ok(()));

        let mngr = DefaultManager { repo: mock_repo };
        assert!(mngr.rm("some", Some("dump")).is_ok());
    }
}
