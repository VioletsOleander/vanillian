use anyhow::Result;

use crate::arg::CommitSubcommand;

mod daily;
mod note;

use daily::CommitDaily;

pub struct Commit;

impl Commit {
    pub fn run(subcommand: &CommitSubcommand) -> Result<()> {
        match subcommand {
            CommitSubcommand::Daily(args) => CommitDaily::run(args),
            _ => Ok(()),
        }
    }
}
