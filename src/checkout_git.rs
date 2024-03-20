use crate::command::Command;
use clap::Parser;
use std::fs;
use std::fs::File;

#[derive(Parser, Debug, Clone)]
pub struct CheckoutArgs {
    branch: String,
    create: Option<bool>,
}

#[derive(Debug)]
pub struct CheckoutCommand {
    arguments: CheckoutArgs,
}

impl CheckoutCommand {
    pub fn new(args: CheckoutArgs) -> Self {
        CheckoutCommand { arguments: args }
    }
}

impl Command for CheckoutCommand {
    fn execute(&mut self) {
        let checkout_ref = "./.pit/refs/".to_string() + &self.arguments.branch.clone();

        let file = File::open(checkout_ref.clone());
        if file.is_err() {
            if self.arguments.create.is_none() {
                return;
            }

            let file_result = File::create(checkout_ref.clone());
            if file_result.is_err() {
                println!("Cannot create branch {:?}", file_result);
                return;
            }
        }

        // if self.arguments.get_current.is_some() {
        //     let head = fs::read_to_string("./.pit/HEAD").unwrap();
        //     println!("{}", head);
        //     let commit_result = fs::read_to_string(".pit/".to_string() + &head);
        //     if commit_result.is_err() {
        //         println!("Cannot copy the current branch {:?}", commit_result);
        //         return;
        //     }
        //     let _ = fs::write(checkout_ref.clone(), commit_result.unwrap());
        // }

        let result = fs::write(
            "./.pit/HEAD",
            "refs/".to_string() + &self.arguments.branch.clone(),
        );

        if result.is_err() {
            println!("Error happened when changing branch {:?}", result);
        }

        let _ = fs::write("./.pit/objects/info", "");

        println!("Changed branch to {}", self.arguments.branch);
    }
}
