use std::process::{Command, Stdio};

use anyhow::{Result, anyhow};
use regex::Regex;

pub struct CommitNote;

struct NoteInfo<'a> {
    category: &'a str,
    identifier: &'a str,
}

impl CommitNote {
    pub fn run() -> Result<()> {
        let note_path = get_note_path()?;

        let note_info = get_note_info(&note_path)?;
        let note_exists = note_exists(&note_path)?;

        let header = make_header(note_info, note_exists)?;

        // `Command.output` will capture stdout, which will cause deadlock:
        // process launch git -> git launch editor -> editor waits for stdout (captured by process)
        // -> git waits for editor to exit -> process waits for git to exit -> deadlock
        // `Command.status` will not capture stdout, therefore deadlock will not happen.
        let status = Command::new("git")
            .args(["commit", "--edit", "-m", &header])
            .status()?;

        match status.code() {
            // Git commit exited normally.
            Some(0) => Ok(()),
            // Git commit aborted by the user.
            Some(1) => Ok(()),
            Some(code) => Err(anyhow!("subprocess failed with return code {code}")),
            None => Err(anyhow!("subprocess terminated by signal.")),
        }
    }
}

/// Return the path of the staged note file.
///
/// The returned path is relative to the root directory of current git repository.
fn get_note_path() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--name-only"])
        .output()?;

    let paths = String::from_utf8(output.stdout)?;
    let re = Regex::new(r"^[a-z]+-notes/.+\.md$")?;
    let mut note_paths = paths.split('\n').filter(|&path| re.is_match(path));

    match (note_paths.next(), note_paths.next()) {
        (Some(path), None) => Ok(path.to_string()),
        (None, None) => Err(anyhow!("no staged note file found")),
        _ => Err(anyhow!("multiple staged note file found")),
    }
}

/// Return true if the note file already exists in HEAD, othrwise return false.
fn note_exists(note_path: &str) -> Result<bool> {
    // Capture stderr so that git cat-file do not print borthering message if file does not exists
    // in HEAD.
    let status = Command::new("git")
        .args(["cat-file", "-e", &format!("HEAD:{note_path}")])
        .stderr(Stdio::null())
        .status()?;

    Ok(status.success())
}

fn get_note_info<'a>(note_path: &'a str) -> Result<NoteInfo<'a>> {
    // Pattern breakdown:
    // ^(?<category>[a-z]+)-notes/ --> capture note category (e.g. "paper" from paper-notes/)
    // (?<identifier>[^/]+?)\.md$  --> capture note identifier (e.g. "rust/cargo/Glossary" from doc-notes/rust/cargo/Glossary.md)
    let re = Regex::new(r"^(?<category>[a-z]+)-notes/(?<identifier>.+?)\.md$")?;

    let captures = re
        .captures(note_path)
        .ok_or_else(|| anyhow!("note path {} does not match the desired pattern", note_path))?;

    let category = captures
        .name("category")
        .ok_or_else(|| {
            anyhow!(
                "failed to capture note category from note path {}",
                note_path
            )
        })?
        .as_str();

    let identifier = captures
        .name("identifier")
        .ok_or_else(|| {
            anyhow!(
                "failed to capture note identifier from note path {}",
                note_path
            )
        })?
        .as_str();

    Ok(NoteInfo {
        category,
        identifier,
    })
}

fn make_header(note_info: NoteInfo, note_exists: bool) -> Result<String> {
    // If the note file already exists in HEAD, it's an update. Otherwise, it's an addition.
    let action = match note_exists {
        true => "update",
        false => "add",
    };

    match note_info.category {
        "paper" => {
            // Pattern breakdown:
            // (?<subidentifier>.+)       -> capture paper subidentifier (e.g. "nlp/foo" from nlp/foo-2026-CONF)
            // (?:-[0-9]{4}(?:-[A-Z]+)?)? -> capture optional -YYYY or -YYYY-CONF
            let re = Regex::new(r"^(?<subidentifier>.+)(?:-[0-9]{4}(?:-[A-Z]+)?)?$")?;
            let captures = re.captures(note_info.identifier).ok_or_else(|| {
                anyhow!(
                    "paper note identifier {} does not match the desired pattern",
                    note_info.identifier
                )
            })?;

            let subidentifier = captures
                .name("subidentifier")
                .ok_or_else(|| {
                    anyhow!(
                        "failed to capture paper subidentifier from paper identifier {}",
                        note_info.identifier
                    )
                })?
                .as_str();

            Ok(format!("note(paper): {action} '{subidentifier}'"))
        }
        "doc" => {
            // Pattern breakdown:
            // (?:(?<subcategory>[^/]+)/)? -> capture optional doc subcategory (e.g. "rust" from rust/cargo/Glossary)
            // (?<subidentifier>.+?)       -> capture doc subidentifier (e.g. "cargo/Glossary" from rust/cargo/Glossary)
            let re = Regex::new(r"^(?:(?<subcategory>[^/]+)/)?(?<subidentifier>.+)$")?;
            let captures = re.captures(note_info.identifier).ok_or_else(|| {
                anyhow!(
                    "doc note identifier {} does not match the desired pattern",
                    note_info.identifier
                )
            })?;

            let subcategory = captures.name("subcategory").map(|m| m.as_str());
            let subidentifier = captures
                .name("subidentifier")
                .ok_or_else(|| {
                    anyhow!(
                        "failed to capture note subidentifier from note identifier {}",
                        note_info.identifier
                    )
                })?
                .as_str();

            match subcategory {
                Some(subcategory) => Ok(format!("doc({subcategory}): {action} '{subidentifier}'")),
                None => Ok(format!("doc: {action} '{subidentifier}'")),
            }
        }
        _ => Ok(format!(
            "note({category}): {action} '{identifier}'",
            category = note_info.category,
            identifier = note_info.identifier,
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn paper_note_info() -> Result<()> {
        let note_path = "paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm-2014-ATC.md";
        let note_info = get_note_info(note_path)?;

        assert_eq!("paper", note_info.category);
        assert_eq!(
            "distributed-system/In Search of an Understandable Consensus Algorithm-2014-ATC",
            note_info.identifier
        );

        Ok(())
    }

    #[test]
    fn doc_note_info() -> Result<()> {
        let note_path = "doc-notes/rust/reference/Crates and source files.md";
        let note_info = get_note_info(note_path)?;

        assert_eq!("doc", note_info.category);
        assert_eq!(
            "rust/reference/Crates and source files",
            note_info.identifier
        );

        Ok(())
    }

    #[test]
    fn paper_note_header() -> Result<()> {
        let note_path = "paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm-2014-ATC.md";
        let note_info = get_note_info(note_path)?;
        let header = make_header(note_info, true)?;

        assert_eq!(
            "note(paper): update 'distributed-system/In Search of an Understandable Consensus Algorithm-2014-ATC'",
            header
        );

        Ok(())
    }

    #[test]
    fn doc_note_header_with_subcategory() -> Result<()> {
        let note_path = "doc-notes/rust/reference/Crates and source files.md";
        let note_info = get_note_info(note_path)?;
        let header = make_header(note_info, true)?;

        assert_eq!(
            "doc(rust): update 'reference/Crates and source files'",
            header
        );

        Ok(())
    }

    #[test]
    fn doc_note_header_without_subcategory() -> Result<()> {
        let note_path = "doc-notes/Semantic Versioning.md";
        let note_info = get_note_info(note_path)?;
        let header = make_header(note_info, true)?;

        assert_eq!("doc: update 'Semantic Versioning'", header);

        Ok(())
    }
}
