mod add_git;
mod checkout_git;
mod command;
mod commit_git;
mod diff;
mod init_git;
mod merge;
mod status_git;

use crate::command::Command;
use clap::{command, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add(add_git::AddArgs),
    Init(init_git::InitArgs),
    Status(status_git::StatusArgs),
    Commit(commit_git::CommitArgs),
    Checkout(checkout_git::CheckoutArgs),
    Diff(diff::DiffArgs),
    Merge(merge::MergeArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add(args) => {
            let mut x = add_git::AddCommand::new(args.clone());
            x.execute();
        }
        Commands::Init(args) => {
            let mut x = init_git::InitializeCommand::new(args.clone());
            x.execute();
        }
        Commands::Status(args) => {
            let mut x = status_git::StatusCommand::new(args.clone());
            x.execute();
        }
        Commands::Commit(args) => {
            let mut x = commit_git::CommitCommand::new(args.clone());
            x.execute();
        }
        Commands::Checkout(args) => {
            let mut x = checkout_git::CheckoutCommand::new(args.clone());
            x.execute();
        }
        Commands::Diff(args) => {
            let mut x = diff::DiffCommand::new(args.clone());
            x.execute();
        }
        Commands::Merge(args) => {
            let mut x = merge::MergeCommand::new(args.clone());
            x.execute();
        }
    };
}
