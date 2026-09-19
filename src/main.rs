use anyhow::Result;
use clap::Parser;

mod arg;
mod command;

use arg::{VanillianArgs, VanillianSubcommand};
use command::{Canonicalize, Commit};

fn main() -> Result<()> {
    let args = VanillianArgs::parse();

    match args.subcommand() {
        VanillianSubcommand::Canonicalize(args) => Canonicalize::run(args)?,
        VanillianSubcommand::Commit(subcommand) => Commit::run(subcommand)?,
    };

    Ok(())
}
