use std::process::Command;

use anyhow::Result;

pub struct CommitNote;

impl CommitNote {
    pub fn run() -> Result<()> {}
}

/// Return the path of the staged note file, or None if no note file is staged.
///
/// The returned path is relative to the root directory of current git repository.
fn get_note_path() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--name-only"])
        .output()?;
}
