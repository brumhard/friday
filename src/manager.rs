use std::{
    collections::HashMap,
    default, fmt,
    fs::{self, File},
    path::{Path, PathBuf},
    str,
};

use crate::{Error, Result};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Section {
    Dump,
    Custom(String),
}

impl default::Default for Section {
    fn default() -> Self {
        Self::Dump
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dump => write!(f, "dump"),
            Self::Custom(x) => write!(f, "{x}"),
        }
    }
}

impl str::FromStr for Section {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "" | "dump" => Ok(Self::Dump),
            any => Ok(Self::Custom(any.to_string())),
        }
    }
}

trait Manager {
    fn add(&self, task: &str, section: Option<&str>) -> Result<()>;
    fn list(&self, section: Option<&str>) -> Result<Vec<String>>;
    fn rm(&self, pattern: &str, section: Option<&str>) -> Result<()>;
    fn manage(&self) -> Result<()> {
        // TODO: this is probably not the right pattern for this
        panic!("this should not be used if not implemented")
    }
}

trait Repo {
    fn create(&self, task: &str, section: Section) -> Result<()>;
    fn list(&self, section: Section) -> Result<Vec<String>>;
    fn delete(&self, pattern: &str, section: Section) -> Result<()>;
}

struct FileBackedRepo<T: AsRef<Path>> {
    file: T,
}

impl<T: AsRef<Path>> FileBackedRepo<T> {
    fn new(path: T) -> Result<FileBackedRepo<T>> {
        Ok(FileBackedRepo { file: path })
    }

    fn open_file(&self) -> Result<File> {
        let file = File::options()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.file)?;
        Ok(file)
    }

    fn sections(&self) -> Result<HashMap<Section, Vec<String>>> {
        let file_content = fs::read_to_string(&self.file)?;
        let valid_lines = file_content
            .lines()
            .map(|line| line.trim())
            .filter(|&line| line.starts_with('-') || line.starts_with("##"));
        let mut sections_to_tasks: HashMap<Section, Vec<String>> = HashMap::new();

        let mut current_section: Section = Default::default();
        for line in valid_lines {
            let content = line
                .split_whitespace()
                .skip(1)
                .collect::<Vec<&str>>()
                .join(" ");

            if line.starts_with("##") {
                current_section = content.parse().unwrap();
                continue;
            }

            sections_to_tasks
                .entry(current_section.clone())
                .or_default()
                .push(content);
        }

        Ok(sections_to_tasks)
    }

    fn filter_for_section(&self, section: Section) -> Result<Vec<String>> {
        let sections = self.sections()?;
        let tasks = sections
            .get(&section)
            .ok_or_else(|| {
                Error::InvalidArgument(format!("section {section} not found").to_string())
            })?
            .to_owned();
        Ok(tasks)
    }
}

impl<T: AsRef<Path>> Repo for FileBackedRepo<T> {
    fn create(&self, task: &str, section: Section) -> Result<()> {
        todo!()
    }

    fn delete(&self, pattern: &str, section: Section) -> Result<()> {
        todo!()
    }

    fn list(&self, section: Section) -> Result<Vec<String>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;
    use std::error::Error;
    use std::vec;
    use std::{io::Write, result::Result};

    // the returned temp_dir is only returned to keep the reference and not destroy it
    // before the function tests are done.
    fn setup(content: &str) -> Result<(FileBackedRepo<PathBuf>, TempDir), Box<dyn Error>> {
        let tmp_dir = tempdir::TempDir::new("example")?;
        let file_path = tmp_dir.path().join("testing");
        let mut tmp_file = File::create(&file_path)?;
        writeln!(tmp_file, "{content}")?;
        let file_repo = FileBackedRepo::new(file_path)?;
        Ok((file_repo, tmp_dir))
    }

    macro_rules! test_sections {
        ($name:ident, $in:expr $(,($key:expr, $value:expr))*) => {
            #[test]
            fn $name() -> Result<(), Box<dyn Error>> {
                let (file_repo, _tmp_dir) = setup($in)?;
                let sections = file_repo.sections()?;
                assert_eq!(
                    sections,
                    HashMap::from([$(
                        ($key, $value.iter().map(|s| s.to_string()).collect()),
                    )*])
                );
                Ok(())
            }
        };
    }

    test_sections!(sections_no_content, "");
    test_sections!(
        sections_only_dump,
        "\
## Dump
- in dump section",
        (Section::Dump, vec!("in dump section"))
    );
    test_sections!(
        sections_multiple,
        "\
## Dump
- in dump section

## Something
- in something",
        (Section::Dump, vec!("in dump section")),
        (
            Section::Custom("something".to_string()),
            vec!("in something")
        )
    );
    test_sections!(
        sections_ignore_toplevel_headings_and_comments,
        "\
# Toplevel heading
## Dump
- in dump section
<!-- This is some comment -->",
        (Section::Dump, vec!("in dump section"))
    );
    test_sections!(
        sections_uses_dump_as_default,
        "\
# Toplevel heading

- this is somewhere in the file
<!-- This is some comment -->",
        (Section::Dump, vec!("this is somewhere in the file"))
    );

    #[test]
    fn filter_for_section_returns_error_on_not_found() {
        let (file_repo, _tmp_dir) = setup("").unwrap();
        assert!(file_repo.filter_for_section(Section::Dump).is_err())
    }

    #[test]
    fn filter_for_section_works() -> Result<(), Box<dyn Error>> {
        let (file_repo, _tmp_dir) = setup("##Dump\n- something")?;
        let items = file_repo.filter_for_section(Section::Dump)?;
        assert_eq!(items, vec!("something".to_string()));
        Ok(())
    }
}
