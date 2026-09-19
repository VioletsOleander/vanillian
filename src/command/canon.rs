use std::fs::{File, rename};
use std::io::{BufRead, BufReader, BufWriter, Write};

use anyhow::{Result, anyhow};
use itertools::process_results;
use regex::{Regex, regex};

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
        let reader = BufReader::new(File::open(args.file())?);

        let f_temp = format!("{}.VANILLIAN.TEMPORARY", args.file());
        let writer = BufWriter::new(File::open(&f_temp)?);

        // The resturn value is a result wraps another result, the outer result is from
        // reader.lines().next(), the inner result is from canonicalize_lines().
        process_results(reader.lines(), |lines| canonicalize_lines(lines, writer))??;

        let f_out = match args.overwrite() {
            true => args.file(),
            false => &format!("{}.canonicalized", args.file()),
        };

        rename(f_temp, f_out)?;

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
    // Match - [[<any>]]: <any> or - [[<any>]]
    let entry_regex = Regex::new(r"^- \[\[(?<identifier>.+)\]\](?<suffix>:.+)?$")?;

    let mut current_kind = SectionKind::Other;
    for line in lines {
        if let Some(section) = sections.iter().find(|&section| section.is_match(&line)) {
            // Current line is a section header.
            current_kind = section.kind;
            write_line(&mut writer, line.trim_end())?;
            continue;
        };

        let captures = entry_regex.captures(&line);

        match captures {
            Some(captures) => {
                // Current line is a log entry.
                let identifier = captures
                    .name("identifier")
                    .ok_or_else(|| anyhow!("failed to capture identifier from entry {}", line))?
                    .as_str();

                // I am suprised to find that this compiles. It relies on "temporary lifetime extension".
                // Apprantely this makes the code much concise.
                let identifier = match current_kind {
                    SectionKind::Doc => &canonicalize_doc_identifier(identifier)?,
                    SectionKind::Wiki => &canonicalize_wiki_identifier(identifier)?,
                    SectionKind::Paper => &canonicalize_paper_identifier(identifier)?,
                    _ => identifier,
                };

                match captures.name("suffix") {
                    Some(suffix) => {
                        let line = format!("- [[{identifier}]]{suffix}", suffix = suffix.as_str());
                        write_line(&mut writer, &line)?;
                    }
                    None => {
                        let line = format!("- [[{identifier}]]");
                        write_line(&mut writer, &line)?;
                    }
                }
            }
            None => {
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

fn canonicalize_doc_identifier(identifier: &str) -> Result<String> {
    let (path, _) = split_identifier(identifier.trim_end())?;

    let subpath = path.strip_prefix("doc-notes/").ok_or_else(|| {
        anyhow!(
            "expected doc path perfixed with doc-notes/, but got {}",
            identifier
        )
    })?;

    Ok(format!("{path}|{subpath}"))
}

fn canonicalize_wiki_identifier(identifier: &str) -> Result<String> {
    let (path, _) = split_identifier(identifier.trim_end())?;

    let subpath = path.strip_prefix("wiki-notes/").ok_or_else(|| {
        anyhow!(
            "expected wiki identifier perfixed with wiki-notes/, but got {}",
            identifier
        )
    })?;

    Ok(format!("{path}|{subpath}"))
}

fn canonicalize_paper_identifier(identifier: &str) -> Result<String> {
    let (path, name) = split_identifier(identifier.trim_end())?;

    // Match <name> or <name>-<year> or <name>-<year>-<publisher>
    // Group <name> should be ungreedy, otherwise capture will fail.
    let re = regex!(r"^(?<name>.+?)(?:-(?<year>[0-9]{4}))?(?:-(?<publisher>[A-Z]+))?$");
    let captures = re
        .captures(name)
        .ok_or_else(|| anyhow!("note name {} does not match the desired pattern", name))?;

    let paper_name = captures
        .name("name")
        .ok_or_else(|| anyhow!("failed to capture paper name from note name {}", name))?
        .as_str();

    let year = captures
        .name("year")
        .ok_or_else(|| {
            anyhow!(
                "failed to capture paper publish year from note name {}",
                name
            )
        })?
        .as_str();

    let publisher = captures
        .name("publisher")
        .ok_or_else(|| anyhow!("failed to capture paper publisher from note name {}", name))?
        .as_str();

    Ok(format!(
        "{path}|{identifier}",
        identifier = [year, publisher, paper_name].join("-")
    ))
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
    fn doc_identifier() -> Result<()> {
        let identifier = "doc-notes/rust/reference/items/Modules|Modules";
        let identifier = canonicalize_doc_identifier(identifier)?;

        assert_eq!(
            identifier,
            "doc-notes/rust/reference/items/Modules|rust/reference/items/Modules",
        );

        Ok(())
    }

    #[test]
    fn wiki_identifier() -> Result<()> {
        let identifier = "wiki-notes/Type variance|Type variance";
        let identifier = canonicalize_wiki_identifier(identifier)?;

        assert_eq!(identifier, "wiki-notes/Type variance|Type variance");

        Ok(())
    }

    #[test]
    fn paper_identifier() -> Result<()> {
        let identifier = "paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC|In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC";

        let identifier = canonicalize_paper_identifier(identifier)?;

        assert_eq!(
            identifier,
            "paper-notes/distributed-system/In Search of an Understandable Consensus Algorithm (Extended Version)-2014-ATC|2014-ATC-In Search of an Understandable Consensus Algorithm (Extended Version)",
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
