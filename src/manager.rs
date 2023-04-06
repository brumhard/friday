use crate::Result;
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
