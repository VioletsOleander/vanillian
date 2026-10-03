use clap::builder::Styles;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
#[command(styles = Styles::styled())]
pub struct VanillianArgs {
    #[command(subcommand)]
    pub subcommand: VanillianSubcommand,
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
    pub file: String,
    #[arg(short, long)]
    /// Overwrite the original file instead of writing to a ".canonicalized" suffixed file.
    pub overwrite: bool,
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
    pub late: bool,
    #[arg(short, long)]
    /// Do not open an editor for editing the commit message further.
    pub skip_edit: bool,
}
