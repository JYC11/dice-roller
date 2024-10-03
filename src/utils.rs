use crate::builders::{build_dice_roll_commands, build_result_keeping_rules, build_success_counting_rules};
use crate::enums::Operator;
use crate::roll_command::InitialDiceRollResult;
use crate::traits::TableDisplay;
use regex::Regex;

pub fn yn_tf_to_bool(value: String) -> bool {
    let lowercased = value.to_lowercase();
    if lowercased == "y" || lowercased == "t" { true } else if lowercased == "n" || lowercased == "f" { false } else { unreachable!() }
}

pub fn parse_operator(input: &str) -> Option<Operator> {
    let number = parse_number(input);

    let res = if input.contains("eq") {
        Operator::Eq(number)
    } else if input.contains("lte") {
        Operator::Lte(number)
    } else if input.contains("gte") {
        Operator::Gte(number)
    } else if input.contains("lt") {
        Operator::Lt(number)
    } else if input.contains("gt") {
        Operator::Gt(number)
    } else {
        unreachable!()
    };

    Some(res)
}

pub fn parse_number(input: &str) -> u32 {
    let number_part = Regex::new(r"\d+").unwrap();
    let number = number_part.find(input)
        .unwrap().as_str().parse().unwrap();
    number
}

pub fn _sample_for_testing() {
    let res = build_dice_roll_commands(
        "10d20".parse().unwrap(),
        None,
        None,
        None,
        None,
    );
    let commands = res.0;
    let modifier = res.1;
    let mut initial_results: Vec<InitialDiceRollResult> = vec![];
    for command in commands {
        initial_results.append(&mut command.roll_dice())
    }
    let result_keeping_rules = build_result_keeping_rules(
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let success_keeping_rules = build_success_counting_rules(
        None,
        Some("lte10".parse().unwrap()),
        None,
        None,
        Some(1),
        None,
        None,
    );
    let mut secondary_results = result_keeping_rules.process_results(&mut initial_results);
    let final_results = success_keeping_rules.count_successes(&mut secondary_results, modifier);
    final_results.display()
}