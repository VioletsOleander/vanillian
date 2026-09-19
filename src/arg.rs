use clap::builder::Styles;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
#[command(styles = Styles::styled())]
pub struct VanillianArgs {
    #[command(subcommand)]
    subcommand: VanillianSubcommand,
}

impl VanillianArgs {
    pub fn subcommand(&self) -> &VanillianSubcommand {
        &self.subcommand
    }
}

#[derive(Subcommand)]
pub enum VanillianSubcommand {
    /// Canonicalize lines in the specified log file.
    Canonicalize(CanonicalizeArgs),
    #[command(subcommand)]
    /// Commit staged changes with generated message header.
    Commit(CommitSubcommand),
}

#[derive(Args)]
pub struct CanonicalizeArgs {
    /// Path to the log file.
    file: String,
    #[arg(short, long)]
    /// Overwrite the original file instead of writing to a ".canonicalized" suffixed file.
    overwrite: bool,
}

impl CanonicalizeArgs {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn overwrite(&self) -> bool {
        self.overwrite
    }
}

#[derive(Subcommand)]
pub enum CommitSubcommand {
    /// Generate a message header for daily commit and commit staged changes.
    Daily(CommitDailyArgs),
    /// Generate a message header based on staged notes and commit staged changes.
    Note,
}

#[derive(Args)]
pub struct CommitDailyArgs {
    #[arg(short, long)]
    /// Generate the message header for yesterday.
    late: bool,
    #[arg(short, long)]
    /// Do not open an editor for editing the commit message further.
    skip_edit: bool,
}

impl CommitDailyArgs {
    pub fn late(&self) -> bool {
        self.late
    }

    pub fn skip_edit(&self) -> bool {
        self.skip_edit
    }
}
