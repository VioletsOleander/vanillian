use anyhow::Result;
use clap::Parser;

mod arg;
mod command;

use arg::{VanillianArgs, VanillianSubcommand};
use command::Commit;

fn main() -> Result<()> {
    let args = VanillianArgs::parse();

    match args.subcommand() {
        VanillianSubcommand::Canonicalize(args) => {}
        VanillianSubcommand::Commit(subcommand) => Commit::run(subcommand)?,
    };

    Ok(())
}
