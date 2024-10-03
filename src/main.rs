extern crate core;

mod roll_result;
mod roll_command;
mod enums;
mod result_keeping_rules;
mod success_counting_rules;
mod traits;
mod cli;
mod utils;
mod builders;


fn main() {
    cli::cli_app();
}
