extern crate core;

use display_logic::cli;

mod enums;
mod utils;

mod dice_rolling_logic;
mod display_logic;

fn main() {
    cli::cli_app();
}
