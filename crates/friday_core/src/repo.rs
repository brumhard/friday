use crate::{error::Result, Error, Section};
use core::fmt;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::str;
use std::{fs, path::Path};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Repo {
    fn create(&self, task: &str, section: Section) -> Result<()>;
    fn list(&self, section: Section) -> Result<Vec<String>>;
    fn list_all(&self) -> Result<HashMap<Section, Vec<String>>>;
    fn delete(&self, task: &str, section: Section) -> Result<()>;
}

pub struct FileBacked<T: AsRef<Path>> {
    file: T,
}

struct Line {
    section: Section,
    content: LineContent,
}

enum LineContent {
    Ignored(String),
    Task(String),
    Section(String),
}

impl LineContent {
    fn stripped(&self) -> String {
        if let LineContent::Ignored(x) = self {
            return x.to_string();
        }

        self.to_string()
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

impl fmt::Display for LineContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ignored(x) | Self::Task(x) | Self::Section(x) => write!(f, "{x}"),
        }
    }
}

impl str::FromStr for LineContent {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim() {
            x if x.starts_with('-') => Ok(Self::Task(s.to_string())),
            x if x.starts_with("##") => Ok(Self::Section(s.to_string())),
            _ => Ok(Self::Ignored(s.to_string())),
        }
    }
}

impl<T: AsRef<Path>> FileBacked<T> {
    pub fn new(path: T) -> Result<FileBacked<T>> {
        let file = File::options().create(true).append(true).open(&path)?;
        if file.metadata()?.len() == 0 {
            writeln!(
                &file,
                "\
# It's friday my dudes

## todo

- start here

<!-- this is a comment ignored by default -->
## dump

- this where stuff lands by default"
            )?;
        }
        Ok(FileBacked { file: path })
    }

    fn lines(&self) -> Result<Vec<Line>> {
        let file_content = fs::read_to_string(&self.file)?;
        let mut current_section = Section::default();
        let mut lines = Vec::new();
        // the use of split instead of lines() is intended to keep the
        // any trailing newline characters in the file
        for line in file_content.split('\n') {
            let content = line.parse().unwrap();
            if matches!(content, LineContent::Section(_)) {
                current_section = content.stripped().parse().unwrap();
            }

            lines.push(Line {
                section: current_section.clone(),
                content,
            });
        }
        Ok(lines)
    }

    fn dump_lines(&self, lines: &[Line]) -> Result<()> {
        let content = lines
            .iter()
            .map(|l| l.content.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        fs::write(&self.file, content)?;
        Ok(())
    }
}

impl<T: AsRef<Path>> Repo for FileBacked<T> {
    fn create(&self, task: &str, section: Section) -> Result<()> {
        let mut lines = self.lines()?;

        let mut last_line_in_section = None;
        let mut first_section_line = None;
        for (i, line) in lines.iter().enumerate() {
            if matches!(line.content, LineContent::Ignored(_)) {
                continue;
            }

            if matches!(line.content, LineContent::Section(_)) && first_section_line.is_none() {
                first_section_line = Some(i);
            }

            if line.section == section {
                last_line_in_section = Some(i);
                continue;
            }

            if last_line_in_section.is_some() {
                break;
            }
        }

        if last_line_in_section.is_none() {
            let section_line = Line {
                section: section.clone(),
                content: LineContent::Section(format!("## {section}")),
            };
            // insert section either before the first other section
            // or at the end if there are no sections yet
            if let Some(line) = first_section_line {
                lines.insert(line, section_line);
                last_line_in_section = Some(line);
            } else {
                lines.push(section_line);
                last_line_in_section = Some(lines.len() - 1);
            }
        }

        // insert task after last_line_in_section
        lines.insert(
            last_line_in_section.unwrap() + 1,
            Line {
                section,
                content: LineContent::Task(format!("- {task}")),
            },
        );

        self.dump_lines(&lines)
    }

    fn delete(&self, task: &str, section: Section) -> Result<()> {
        let mut lines = self.lines()?;
        let mut remove_index = None;
        for (i, line) in lines.iter().enumerate() {
            if !matches!(line.content, LineContent::Task(_)) || line.section != section {
                continue;
            }

            if line.content.stripped() == task {
                remove_index = Some(i);
                break;
            }
        }
        if remove_index.is_none() {
            return Ok(());
        }

        lines.remove(remove_index.unwrap());
        self.dump_lines(&lines)
    }

    fn list(&self, section: Section) -> Result<Vec<String>> {
        let sections = self.list_all()?;
        let tasks = sections
            .get(&section)
            .ok_or_else(|| Error::InvalidArgument(format!("section {section} not found")))?
            .clone();
        Ok(tasks)
    }

    fn list_all(&self) -> Result<HashMap<Section, Vec<String>>> {
        let mut sections_to_tasks: HashMap<Section, Vec<String>> = HashMap::new();
        let lines = self.lines()?;
        let task_lines = lines
            .iter()
            .filter(|l| matches!(l.content, LineContent::Task(_)));
        for line in task_lines {
            sections_to_tasks
                .entry(line.section.clone())
                .or_default()
                .push(line.content.stripped());
        }
        Ok(sections_to_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::path::PathBuf;
    use std::result::Result;
    use std::vec;
    use tempdir::TempDir;

    #[test]
    fn new_creates_file_with_header_if_not_exists() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempdir::TempDir::new("example")?;
        let file_path = tmp_dir.path().join("testing");
        FileBacked::new(&file_path)?;
        assert!(file_path.exists());
        assert!(fs::read_to_string(file_path)?.contains("friday"));
        Ok(())
    }

    // the returned temp_dir is only returned to keep the reference and not destroy it
    // before the function tests are done.
    fn setup(content: &str) -> Result<(FileBacked<PathBuf>, TempDir), Box<dyn Error>> {
        let tmp_dir = tempdir::TempDir::new("example")?;
        let file_path = tmp_dir.path().join("testing");
        fs::write(&file_path, content)?;
        let file_repo = FileBacked::new(file_path)?;
        Ok((file_repo, tmp_dir))
    }

    macro_rules! test_list_all {
        ($name:ident, $in:expr $(,($key:expr, $value:expr))*) => {
            #[test]
            fn $name() -> Result<(), Box<dyn Error>> {
                let (file_repo, _tmp_dir) = setup($in)?;
                let sections = file_repo.list_all()?;
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

    test_list_all!(
        list_all_no_content,
        "",
        (Section::Dump, vec!["this where stuff lands by default"]),
        (Section::Custom("todo".to_string()), vec!["start here"])
    );
    test_list_all!(
        list_all_only_dump,
        "\
## Dump
- in dump section",
        (Section::Dump, vec!("in dump section"))
    );
    test_list_all!(
        list_all_multiple,
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
    test_list_all!(
        list_all_ignore_toplevel_headings_and_comments,
        "\
# Toplevel heading
## Dump
- in dump section
<!-- This is some comment -->",
        (Section::Dump, vec!("in dump section"))
    );
    test_list_all!(
        list_all_uses_dump_as_default,
        "\
# Toplevel heading

- this is somewhere in the file
<!-- This is some comment -->",
        (Section::Dump, vec!("this is somewhere in the file"))
    );
    test_list_all!(
        list_all_ignores_whitespace,
        "       - this is somewhere in the file",
        (Section::Dump, vec!("this is somewhere in the file"))
    );

    #[test]
    fn list_returns_error_on_not_found() {
        let (file_repo, _tmp_dir) = setup("").unwrap();
        assert!(file_repo
            .list(Section::Custom("non-existent".to_string()))
            .is_err())
    }

    #[test]
    fn list_works() -> Result<(), Box<dyn Error>> {
        let (file_repo, _tmp_dir) = setup("## Dump\n- something")?;
        let items = file_repo.list(Section::Dump)?;
        assert_eq!(items, vec!("something".to_string()));
        Ok(())
    }

    #[test]
    fn no_change_on_lines_and_dump_lines() -> Result<(), Box<dyn Error>> {
        let initial_content = "## Dump\n- something\n";
        let (file_repo, _tmp_dir) = setup(initial_content)?;
        let content = fs::read_to_string(&file_repo.file)?;
        assert_eq!(initial_content, content);

        let lines = file_repo.lines()?;
        file_repo.dump_lines(&lines)?;

        let content = fs::read_to_string(&file_repo.file)?;
        assert_eq!(initial_content, content);
        Ok(())
    }

    struct RepoTest<'a> {
        initial: &'a str,
        new_task: &'a str,
        section: Section,
        expected: &'a str,
    }
    macro_rules! test_repo {
        ($method:ident, $name:ident, $tt:expr) => {
            #[test]
            fn $name() -> Result<(), Box<dyn Error>> {
                let (file_repo, _tmp_dir) = setup($tt.initial)?;
                file_repo.$method($tt.new_task, $tt.section)?;
                let content = fs::read_to_string(file_repo.file)?;
                assert_eq!(content, $tt.expected);
                Ok(())
            }
        };
    }
    macro_rules! test_create {
        ($name:ident, $tt:expr) => {
            test_repo!(create, $name, $tt);
        };
    }
    macro_rules! test_delete {
        ($name:ident, $tt:expr) => {
            test_repo!(delete, $name, $tt);
        };
    }

    test_create!(
        create_adds_to_existing_section,
        RepoTest {
            initial: "## Dump\n- something",
            new_task: "something else",
            section: Section::Dump,
            expected: "## Dump\n- something\n- something else",
        }
    );

    test_create!(
        create_adds_new_section,
        RepoTest {
            initial: "## Dump\n- something",
            new_task: "something else",
            section: Section::Custom("else".to_string()),
            expected: "## else\n- something else\n## Dump\n- something",
        }
    );

    test_create!(
        create_adds_initial_section,
        RepoTest {
            initial: "# This is just a heading",
            new_task: "something else",
            section: Section::Custom("else".to_string()),
            expected: "# This is just a heading\n## else\n- something else",
        }
    );
    test_delete!(
        delete_works,
        RepoTest {
            initial: "## Dump\n- something\n- something else\n",
            new_task: "something else",
            section: Section::Dump,
            expected: "## Dump\n- something\n",
        }
    );
    test_delete!(
        delete_doesnt_delete_anything_else,
        RepoTest {
            initial: "## Dump\n- something\n- something else\n",
            new_task: "something not in the file",
            section: Section::Dump,
            expected: "## Dump\n- something\n- something else\n",
        }
    );
}
