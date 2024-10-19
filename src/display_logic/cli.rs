use crate::display_logic::builders::{
    build_dice_roll_commands, build_result_keeping_rules, build_success_counting_rules,
};
use crate::dice_rolling_logic::roll_command::InitialDiceRollResult;
use clap::Parser;
use regex::Regex;
use crate::utils::TableDisplay;

#[derive(Parser)]
#[command(
    version = "v0.0.1",
    about = "rolls dice",
    long_about = "this rolls dice, whaddaya want more bub"
)]
#[command(next_line_help = true)]
pub struct Cli {
    #[
    arg(
            short,
            value_parser = validate_dice_roll,
            help = "example: dice-roller -d 1d20+7"
    )
    ]
    dice_roll: Option<String>,

    #[
    arg(
            short,
            value_parser = validate_comparison,
            help = "example: dice-roller -d 2d6+9 -r eq1"
    )
    ]
    re_roll: Option<String>, // eq/gt/lt/lte/gte + num

    #[
    arg(
            long = "rr",
            value_parser = validate_yn_tf,
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "y",
            help = "example: dice-roller -d 2d6+9 -r lte4 --rr"
    )
    ]
    re_roll_recursively: Option<String>, // y/n/t/f or Y/N/T/F

    #[
    arg(
            short,
            value_parser = validate_comparison,
            help = "example: dice-roller -d 3d8 -e eq8"
    )
    ]
    xplode: Option<String>, // eq/gt/lt/lte/gte + num

    #[
    arg(
            long = "xo",
            value_parser = validate_yn_tf,
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "n",
            help = "example: dice-roller -d 3d8 -e eq8 --eo"
    )
    ]
    explode_once: Option<String>,

    #[
    arg(
            long = "kh",
            value_parser = clap::value_parser!(u32).range(1..9999),
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "1",
            help = "example: dice-roller -d 2d20+12 --kh 1 OR dice-roller -d 2d20+12 --kh"
    )
    ]
    keep_high: Option<u32>,

    #[
    arg(
            long = "kl",
            value_parser = clap::value_parser!(u32).range(1..9999),
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "1",
            help = "example: dice-roller -d 2d20+12 --kl 1 OR dice-roller -d 2d20+12 --kl"
    )
    ]
    keep_low: Option<u32>,

    #[
    arg(
            long = "dh",
            value_parser = clap::value_parser!(u32).range(1..9999),
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "1",
            help = "example: dice-roller -d 5d10 --dh 2 OR dice-roller -d 2d20+12 --dh"
    )
    ]
    drop_high: Option<u32>,

    #[
    arg(
            long = "dl",
            value_parser = clap::value_parser!(u32).range(1..9999),
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "1",
            help = "example: dice-roller -d 4d6 --dl 1 OR dice-roller -d 2d20+12 --dl",
    )
    ]
    drop_low: Option<u32>,

    #[
    arg(
            long = "max",
            value_parser = clap::value_parser!(u32).range(1..100),
            help = "example: dice-roller -d 1d20+9 --max 15",
    )
    ]
    max: Option<u32>,

    #[
    arg(
            long = "min",
            value_parser = clap::value_parser!(u32).range(1..100),
            help = "example: dice-roller -d 1d20+9 --min 10"
    )
    ]
    min: Option<u32>,

    #[
    arg(
            long = "cs",
            value_parser = validate_comparison,
            help = "example: dice-roller -d 1d20+9 --cs gt19"
    )
    ]
    count_success: Option<String>, // eq/gt/lt/lte/gte + num

    #[
    arg(
            long = "cf",
            value_parser = validate_comparison,
            help = "example: dice-roller -d 1d20+9 --cf lt19"
    )
    ]
    count_failure: Option<String>, // eq/gt/lt/lte/gte + num

    #[
    arg(
            long = "even",
            value_parser = validate_yn_tf,
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "y",
            help = "example: dice-roller -d 10d20 --even"
    )
    ]
    even: Option<String>, // y/n/t/f or Y/N/T/F

    #[
    arg(
            long = "odd",
            value_parser = validate_yn_tf,
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "y",
            help = "example: dice-roller -d 10d20 --odd"
    )
    ]
    odd: Option<String>, // y/n/t/f or Y/N/T/F

    #[
    arg(
            long = "df",
            value_parser = clap::value_parser!(u32).range(1..9999),
            num_args = 0..=1,
            require_equals = true,
            default_missing_value = "1",
            help = "example: dice-roller -d 10d20 --cf lt10 --df 1 OR dice-roller -d 10d20 --cf lt10 --df"
    )
    ]
    deduct_failure: Option<u32>,

    #[
    arg(
            long = "sf",
            value_parser = validate_comparison,
            help = "example: dice-roller -d 10d20 --sf lte10"
    )
    ]
    subtract_failures: Option<String>, // eq/gt/lt/lte/gte + num

    #[
    arg(
            long = "ms",
            value_parser = clap::value_parser!(u32).range(1..9999),
            help = "example: dice-roller -d 1d20+15 --ms 10"
    )
    ]
    margin_of_success: Option<u32>,
}

fn validate_dice_roll(s: &str) -> Result<String, String> {
    let dice_regex = Regex::new(r"\b\d+d\d+([+-]\d+)?\b").unwrap();
    if dice_regex.is_match(s) {
        Ok(s.parse::<String>().unwrap())
    } else {
        Err(
            "Incorrect dice roll format. Correct examples: 1d20+5, 2d6, 1d4-1"
                .parse()
                .unwrap(),
        )
    }
}

fn validate_comparison(s: &str) -> Result<String, String> {
    let comparison_regex = Regex::new(r"\b(eq|lt|lte|gt|gte)\d+\b").unwrap();
    if comparison_regex.is_match(&s.to_lowercase()) {
        Ok(s.parse::<String>().unwrap())
    } else {
        Err(
            "Incorrect number comparison format. Correct examples: eq10, lt10, gt10, gte10, lte10"
                .parse()
                .unwrap(),
        )
    }
}

fn validate_yn_tf(s: &str) -> Result<String, String> {
    let lowercased = s.to_lowercase();
    if lowercased == "y" || lowercased == "n" || lowercased == "t" || lowercased == "f" {
        Ok(s.parse::<String>().unwrap())
    } else {
        Err("Only Y/N/T/F or y/n/t/f is allowed".parse().unwrap())
    }
}

pub fn cli_app() {
    let cli = Cli::parse();

    match cli.dice_roll {
        None => {
            println!("please enter a dice roll or enter -h or --help for details and examples")
        }
        Some(dice_roll) => {
            let res = build_dice_roll_commands(
                dice_roll,
                cli.re_roll,
                cli.re_roll_recursively,
                cli.xplode,
                cli.explode_once,
            );
            let result_keeping_rules = build_result_keeping_rules(
                cli.keep_high,
                cli.keep_low,
                cli.drop_high,
                cli.drop_low,
                cli.max,
                cli.min,
            );
            let success_keeping_rules = build_success_counting_rules(
                cli.count_success,
                cli.count_failure,
                cli.even,
                cli.odd,
                cli.deduct_failure,
                cli.subtract_failures,
                cli.margin_of_success,
            );
            let commands = res.0;
            let modifier = res.1;
            let mut initial_results: Vec<InitialDiceRollResult> = vec![];
            for command in commands {
                initial_results.append(&mut command.roll_dice())
            }
            let mut secondary_results = result_keeping_rules.process_results(&mut initial_results);
            let final_results =
                success_keeping_rules.count_successes(&mut secondary_results, modifier);
            final_results.display()
        }
    }
}
