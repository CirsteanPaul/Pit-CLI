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
fn print_len(s: String) {
    println!("Size is {}", s.len());
}
trait Vehicle {
    fn get_name(&self) -> &str;

}
trait Color {
    fn get_color(&self) -> &str;
}
trait Car: Vehicle + Color {
    fn get_speed(&self) -> u32;

}
#[derive(Copy, Clone, Eq)]
struct Dacia {}

impl dyn Clone {

}

impl Car for Dacia {
    fn get_speed(&self) -> u32 {
        todo!()
    }
    fn get_color(&self) -> &str {

    }
}
fn main() {
    fn cakl(){}
    cakl();
    println!("{}", std::mem::size_of::<dyn Name>());
    let s = String::from("abc");
    print_len(&s);
    s.len();
    return;
    let x = b'a';
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
