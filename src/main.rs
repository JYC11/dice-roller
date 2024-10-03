extern crate core;

mod builders;
mod cli;
mod enums;
mod result_keeping_rules;
mod roll_command;
mod roll_result;
mod success_counting_rules;
mod traits;
mod utils;

fn main() {
    cli::cli_app();
}
