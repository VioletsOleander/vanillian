use anyhow::Result;

use crate::arg::CommitSubcommand;

mod daily;
mod note;

use daily::CommitDaily;
use note::CommitNote;

pub struct Commit;

impl Commit {
    pub fn run(subcommand: &CommitSubcommand) -> Result<()> {
        match subcommand {
            CommitSubcommand::Daily(args) => CommitDaily::run(args),
            CommitSubcommand::Note => CommitNote::run(),
        }
    }
}
