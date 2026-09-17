use std::process::Command;

use anyhow::{Result, anyhow};
use chrono::{Days, Local};

use crate::arg::CommitDailyArgs;

pub struct CommitDaily;

impl CommitDaily {
    pub fn run(args: &CommitDailyArgs) -> Result<()> {
        let header = make_header(args.late());

        let command_args = match args.skip_edit() {
            true => ["commit", "--allow-empty", "--no-edit", "-m", &header],
            false => ["commit", "--allow-empty", "--edit", "-m", &header],
        };

        // `Command.output` will capture stdout, which will cause deadlock:
        // process launch git -> git launch editor -> editor waits for stdout (captured by process)
        // -> git waits for editor to exit -> process waits for git to exit -> deadlock
        // `Command.status` will not capture stdout, therefore deadlock will not happen.
        let status = Command::new("git").args(command_args).status()?;

        match status.code() {
            // Git commit exited normally.
            Some(0) => Ok(()),
            // Git commit aborted by the user.
            Some(1) => Ok(()),
            Some(code) => Err(anyhow!(format!(
                "subprocess failed with return code {code}"
            ))),
            None => Err(anyhow!("subprocess terminated by signal.")),
        }
    }
}

fn make_header(late: bool) -> String {
    let today = Local::now().date_naive();
    let commit_day = match late {
        true => today
            .checked_sub_days(Days::new(1))
            .expect("The result date should be within the range."),
        false => today,
    };

    let date = commit_day.format("%Y-%m-%d");
    let weekday = commit_day.format("%A");

    format!("log(daily) {date} {weekday}")
}
