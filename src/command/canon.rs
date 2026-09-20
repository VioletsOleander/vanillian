use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use anyhow::{Context, Result, anyhow};
use itertools::process_results;
use regex::{Regex, regex};
use tempfile::NamedTempFile;

use crate::arg::CanonicalizeArgs;

pub struct Canonicalize;

#[derive(Clone, Copy, Debug)]
enum SectionKind {
    Paper,
    Doc,
    Wiki,
    Other,
}

struct Section {
    kind: SectionKind,
    regex: Regex,
}

impl Canonicalize {
    pub fn run(args: &CanonicalizeArgs) -> Result<()> {
        let reader = BufReader::new(
            File::open(args.file())
                .with_context(|| format!("failed to open file {}", args.file()))?,
        );

        let f_temp = NamedTempFile::new().context("failed to create temporary file")?;
        let writer = BufWriter::new(&f_temp);

        // The resturn value is a result wraps another result, the outer result is from
        // reader.lines().next(), the inner result is from canonicalize_lines().
        process_results(reader.lines(), |lines| canonicalize_lines(lines, writer))??;

        let f_out = match args.overwrite() {
            true => args.file(),
            false => &format!("{}.canonicalized", args.file()),
        };

        f_temp
            .persist(f_out)
            .with_context(|| format!("failed to persit temp file to {}", f_out))?;

        Ok(())
    }
}

impl SectionKind {
    const fn num_kinds() -> usize {
        4
    }
}

impl Section {
    fn is_match(&self, content: &str) -> bool {
        self.regex.is_match(content)
    }
}

fn canonicalize_lines(lines: impl Iterator<Item = String>, mut writer: impl Write) -> Result<()> {
    let sections = make_sections()?;

    let mut section_kind = SectionKind::Other;
    for line in lines {
        if let Some(section) = sections.iter().find(|&section| section.is_match(&line)) {
            // Current line is a section header.
            section_kind = section.kind;
            write_line(&mut writer, line.trim_end())?;
            continue;
        };

        match line.starts_with("- [[") {
            true => {
                // Current line is a log entry.
                let line = canonicalize_entry(line, section_kind)?;
                write_line(&mut writer, line.trim_end())?;
            }
            false => {
                // Current line is a normal text line.
                write_line(&mut writer, line.trim_end())?;
            }
        }
    }

    writer.flush()?;

    Ok(())
}

fn make_sections() -> Result<[Section; SectionKind::num_kinds()]> {
    // Match \[Doc\], \[Paper\], \[Wiki\], \[<Any>\].
    Ok([
        Section {
            kind: SectionKind::Paper,
            regex: Regex::new(r"^\\\[Paper\\\]\s*$")?,
        },
        Section {
            kind: SectionKind::Doc,
            regex: Regex::new(r"^\\\[Doc\\\]\s*$")?,
        },
        Section {
            kind: SectionKind::Wiki,
            regex: Regex::new(r"^\\\[Wiki\\\]\s*$")?,
        },
        Section {
            kind: SectionKind::Other,
            regex: Regex::new(r"^\\\[.+\\\]\s*$")?,
        },
    ])
}

fn write_line(writer: &mut impl Write, line: &str) -> Result<()> {
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\n")?;

    Ok(())
}

fn canonicalize_entry(entry: String, section_kind: SectionKind) -> Result<String> {
    // Match - [[<any>]]: <any> or - [[<any>]]
    let re = regex!(r"^- \[\[(?<identifier>.+)\]\](?<suffix>:.+)?$");
    let captures = re
        .captures(&entry)
        .ok_or_else(|| anyhow!("entry {} does not match the desired pattern", entry))?;

    let identifier = captures
        .name("identifier")
        .ok_or_else(|| anyhow!("failed to capture identifier from entry {}", entry))?
        .as_str();

    let (path, name) = split_identifier(identifier.trim_end())?;
    let canonical_name = match section_kind {
        SectionKind::Doc => make_doc_name(path)?,
        SectionKind::Wiki => make_wiki_name(path)?,
        SectionKind::Paper => &make_paper_name(path)?,
        _ => name,
    };

    match name == canonical_name {
        true => Ok(entry),
        false => match captures.name("suffix") {
            Some(suffix) => Ok(format!(
                "- [[{path}|{canonical_name}]]{suffix}",
                suffix = suffix.as_str()
            )),
            None => Ok(format!("- [[{path}|{canonical_name}]]")),
        },
    }
}

fn split_identifier(identifier: &str) -> Result<(&str, &str)> {
    let mut parts = identifier.split('|');

    match (parts.next(), parts.next(), parts.next()) {
        (Some(path), Some(name), None) => Ok((path, name)),
        _ => Err(anyhow!(
            "expected identifier has a single | as separator in the middle, but got {}",
            identifier
        )),
    }
}

fn make_doc_name(path: &str) -> Result<&str> {
    let canonical_name = path.strip_prefix("doc-notes/").ok_or_else(|| {
        anyhow!(
            "expected doc path perfixed with doc-notes/, but got {}",
            path
        )
    })?;

    Ok(canonical_name)
}

fn make_wiki_name(path: &str) -> Result<&str> {
    let canonical_name = path.strip_prefix("wiki-notes/").ok_or_else(|| {
        anyhow!(
            "expected wiki path perfixed with wiki-notes/, but got {}",
            path
        )
    })?;

    Ok(canonical_name)
}

fn make_paper_name(path: &str) -> Result<String> {
    // Match <name> or <name>-<year> or <name>-<year>-<publisher>
    // Group <name> should be ungreedy, otherwise capture will fail.
    let re = regex!(
        r"^(?:paper-notes/(?:.*/)?)(?<name>.+?)(?:-(?<year>[0-9]{4}))?(?:-(?<publisher>[A-Z]+))?$"
    );
    let captures = re
        .captures(path)
        .ok_or_else(|| anyhow!("note path {} does not match the desired pattern", path))?;

    let paper_name = captures
        .name("name")
        .ok_or_else(|| anyhow!("failed to capture paper name from note path {}", path))?
        .as_str();

    let year = captures
        .name("year")
        .ok_or_else(|| {
            anyhow!(
                "failed to capture paper publish year from note path {}",
                path
            )
        })?
        .as_str();

    let publisher = captures
        .name("publisher")
        .ok_or_else(|| anyhow!("failed to capture paper publisher from note path {}", path))?
        .as_str();

    Ok([year, publisher, paper_name].join("-"))
}

#[cfg(test)]
mod test {
    use std::io;

    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Default)]
    struct SpyWritier {
        content: String,
    }

    impl Write for SpyWritier {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let content = String::from_utf8(buf.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.content.push_str(&content);

            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn doc_path() -> Result<()> {
        let path = "doc-notes/rust/reference/items/Modules";
        let name = make_doc_name(path)?;

        assert_eq!(name, "rust/reference/items/Modules",);

        Ok(())
    }

    #[test]
    fn wiki_path() -> Result<()> {
        let path = "wiki-notes/software-test/Test double";
        let name = make_wiki_name(path)?;

        assert_eq!(name, "software-test/Test double");

        Ok(())
    }

    #[test]
    fn paper_path() -> Result<()> {
        let path = "paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC";
        let name = make_paper_name(path)?;

        assert_eq!(
            name,
            "2014-ATC-In Search of an Understandable Consensus Algorithm (Extended Version)",
        );

        Ok(())
    }

    #[test]
    fn doc_entry() -> Result<()> {
        let entry = "- [[doc-notes/rust/reference/items/Modules|Modules]]".to_string();
        let canonical_entry = canonicalize_entry(entry, SectionKind::Doc)?;

        assert_eq!(
            canonical_entry,
            "- [[doc-notes/rust/reference/items/Modules|rust/reference/items/Modules]]"
        );

        Ok(())
    }

    #[test]
    fn wiki_entry() -> Result<()> {
        let entry = "- [[wiki-notes/software-test/Test double|Test double]]".to_string();
        let canonical_entry = canonicalize_entry(entry, SectionKind::Wiki)?;

        assert_eq!(
            canonical_entry,
            "- [[wiki-notes/software-test/Test double|software-test/Test double]]"
        );

        Ok(())
    }

    #[test]
    fn paper_entry() -> Result<()> {
        let entry = "- [[paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC|In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC]]".to_string();
        let canonical_entry = canonicalize_entry(entry, SectionKind::Paper)?;

        assert_eq!(
            canonical_entry,
            "- [[paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC|2014-ATC-In Search of an Understandable Consensus Algorithm (Extended Version)]]"
        );

        Ok(())
    }

    #[test]
    fn doc_canonical_entry() -> Result<()> {
        let entry =
            "- [[doc-notes/rust/reference/items/Modules|rust/reference/items/Modules]]".to_string();
        let canonical_entry = canonicalize_entry(entry, SectionKind::Doc)?;

        assert_eq!(
            canonical_entry,
            "- [[doc-notes/rust/reference/items/Modules|rust/reference/items/Modules]]"
        );

        Ok(())
    }

    #[test]
    fn wiki_canonical_entry() -> Result<()> {
        let entry =
            "- [[wiki-notes/software-test/Test double|software-test/Test double]]".to_string();
        let canonical_entry = canonicalize_entry(entry, SectionKind::Wiki)?;

        assert_eq!(
            canonical_entry,
            "- [[wiki-notes/software-test/Test double|software-test/Test double]]"
        );

        Ok(())
    }

    #[test]
    fn paper_canonical_entry() -> Result<()> {
        let entry = "- [[paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC|2014-ATC-In Search of an Understandable Consensus Algorithm (Extended Version)]]".to_string();
        let canonical_entry = canonicalize_entry(entry, SectionKind::Paper)?;

        assert_eq!(
            canonical_entry,
            "- [[paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC|2014-ATC-In Search of an Understandable Consensus Algorithm (Extended Version)]]"
        );

        Ok(())
    }

    #[test]
    fn doc_lines() -> Result<()> {
        let lines = r"
\[Doc\]
- [[doc-notes/rust/api-guidelines/Naming|Naming]]
    Conversion operation should be provided as methods. The name of such method should has following prefixes:
    - `as_` for free conversion, must be borrowed to borrowed
    - `to_` for expensive conversion
    - `into_` for owned to owned conversion
    If getter has runtime checking logic, it is recommended to provide a `_unchecked` suffixed alternative.
".trim_start();

        let canonical_lines = r"
\[Doc\]
- [[doc-notes/rust/api-guidelines/Naming|rust/api-guidelines/Naming]]
    Conversion operation should be provided as methods. The name of such method should has following prefixes:
    - `as_` for free conversion, must be borrowed to borrowed
    - `to_` for expensive conversion
    - `into_` for owned to owned conversion
    If getter has runtime checking logic, it is recommended to provide a `_unchecked` suffixed alternative.
".trim_start();

        let mut writer = SpyWritier::default();
        canonicalize_lines(lines.lines().map(|s| s.to_string()), &mut writer)?;

        assert_eq!(writer.content, canonical_lines);

        Ok(())
    }

    #[test]
    fn wiki_lines() -> Result<()> {
        let lines = r"
\[Wiki\]
- [[wiki-notes/software-test/Test double|Test double]]
    Test double provide fake dependency via interfaces defined in the production code. Therefore, the production code must define well-formed interfaces it expect in order to utilize test double.
    Test doubles are categorized into: stub, mock, fake, spy, dummy
".trim_start();

        let canonical_lines = r"
\[Wiki\]
- [[wiki-notes/software-test/Test double|software-test/Test double]]
    Test double provide fake dependency via interfaces defined in the production code. Therefore, the production code must define well-formed interfaces it expect in order to utilize test double.
    Test doubles are categorized into: stub, mock, fake, spy, dummy
".trim_start();

        let mut writer = SpyWritier::default();
        canonicalize_lines(lines.lines().map(|s| s.to_string()), &mut writer)?;

        assert_eq!(writer.content, canonical_lines);

        Ok(())
    }

    #[test]
    fn paper_lines() -> Result<()> {
        let lines = r"
\[Paper\]
- [[paper-notes/vision/Auto-Encoding Variational Bayes-2014-ICLR|Auto-Encoding Variational Bayes-2014-ICLR]]
    Abstract
        With continuous latent variable, it is intractable to calculate the precise posterior
        distribution for latent variables because of the partition function.
".trim_start();

        let canonical_lines = r"
\[Paper\]
- [[paper-notes/vision/Auto-Encoding Variational Bayes-2014-ICLR|2014-ICLR-Auto-Encoding Variational Bayes]]
    Abstract
        With continuous latent variable, it is intractable to calculate the precise posterior
        distribution for latent variables because of the partition function.
".trim_start();

        let mut writer = SpyWritier::default();
        canonicalize_lines(lines.lines().map(|s| s.to_string()), &mut writer)?;

        assert_eq!(writer.content, canonical_lines);

        Ok(())
    }
}
