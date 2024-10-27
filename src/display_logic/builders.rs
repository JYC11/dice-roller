use crate::dice_rolling_logic::result_keeping_rules::ResultKeepingRules;
use crate::dice_rolling_logic::roll_command::DiceRollCommand;
use crate::dice_rolling_logic::success_counting_rules::SuccessCountingRules;
use crate::enums::Operator;
use crate::utils::{parse_number, parse_operator, yn_tf_to_bool};
use regex::Regex;

pub fn parse_dice(dice_expr: &str) -> (u32, u32) {
    let re = Regex::new(r"(\d+)d(\d+)").unwrap();
    if let Some(caps) = re.captures(dice_expr) {
        let count = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let sides = caps.get(2).unwrap().as_str().parse::<u32>().unwrap();
        (count, sides)
    } else {
        panic!("Invalid dice expression: {}", dice_expr);
    }
}

pub fn build_dice_roll_commands(
    dice_roll: String,
    re_roll: Option<String>,
    re_roll_recursively: Option<String>,
    explode: Option<String>,
    explode_once: Option<String>,
) -> (Vec<DiceRollCommand>, i32) {
    let re_roll_number: u32;
    let re_roll_input = match re_roll {
        None => { re_roll_number = 0; None },
        Some(input) => {
            re_roll_number = parse_number(&input);
            parse_operator(&input)
        }
    };

    let re_roll_recursively_input = match re_roll_recursively {
        None => false,
        Some(value) => yn_tf_to_bool(value),
    };

    let explode_number: u32;
    let explode_input = match explode {
        None => { explode_number = 0; None },
        Some(input) => {
            explode_number = parse_number(&input);
            parse_operator(&input)
        }
    };

    let explode_once_input = match explode_once {
        None => false,
        Some(value) => yn_tf_to_bool(value),
    };

    // This regex matches both dice expressions and numeric modifiers
    let re = Regex::new(r"([+-]?\d+d\d+)|([+-]?\d+)").unwrap();
    let mut result = vec![];
    let mut modifier: i32 = 0;
    let mut group: i32 = 1;
    for caps in re.captures_iter(&dice_roll.to_lowercase()) {
        let token = caps.get(0).unwrap().as_str();
        if token.contains('d') {
            let sign = if token.starts_with('-') { -1 } else { 1 };
            let clean_token = token.trim_start_matches(['+', '-']);
            let count_and_sides = parse_dice(clean_token);
            if re_roll_number > count_and_sides.1 {
                panic!("re-roll number exceeds maximum dice size")
            }
            if explode_number > count_and_sides.1 {
                panic!("explode number exceeds maximum dice size")
            }
            result.push(DiceRollCommand::new(
                group,
                sign,
                count_and_sides.0,
                count_and_sides.1,
                re_roll_input,
                re_roll_recursively_input,
                explode_input,
                explode_once_input,
            ));
            group += 1;
        } else {
            modifier = token.parse::<i32>().unwrap()
        }
    }
    (result, modifier)
}

pub fn build_result_keeping_rules(
    keep_high: Option<u32>,
    keep_low: Option<u32>,
    drop_high: Option<u32>,
    drop_low: Option<u32>,
    max: Option<u32>,
    min: Option<u32>,
) -> ResultKeepingRules {
    let count_keeping_options = [
        keep_high.is_some(),
        keep_low.is_some(),
        drop_high.is_some(),
        drop_low.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();
    assert!(
        count_keeping_options <= 1,
        "Only one of keep_high, keep_low, drop_high, drop_low can be used"
    );

    let count_min_max_options = [max.is_some(), min.is_some()]
        .iter()
        .filter(|&&x| x)
        .count();
    assert!(
        count_min_max_options <= 1,
        "Only one of max or min can be used"
    );

    let keeping_rule = keep_high
        .map(|value| ("keep_high", value))
        .or_else(|| keep_low.map(|value| ("keep_low", value)))
        .or_else(|| drop_high.map(|value| ("drop_high", value)))
        .or_else(|| drop_low.map(|value| ("drop_low", value)))
        .or(None);

    let keep_input: bool;
    let high_input: bool;
    let keep_or_drop_count_input: u32;

    if let Some((rule, value)) = keeping_rule {
        if rule.contains("keep") {
            keep_input = true;
        } else {
            keep_input = false;
        }
        if rule.contains("high") {
            high_input = true;
        } else {
            high_input = false;
        }
        keep_or_drop_count_input = value
    } else {
        keep_input = false;
        high_input = false;
        keep_or_drop_count_input = 0;
    }

    let be_replaced_with_input: Option<u32>;
    let min_input: bool;

    let min_max_rule = min
        .map(|value| ("min", value))
        .or_else(|| max.map(|value| ("max", value)))
        .or(None);

    if let Some((rule, value)) = min_max_rule {
        if rule == "min" {
            min_input = true;
        } else {
            min_input = false;
        }
        be_replaced_with_input = Some(value)
    } else {
        be_replaced_with_input = None;
        min_input = false;
    }

    ResultKeepingRules::new(
        keep_input,
        high_input,
        keep_or_drop_count_input,
        be_replaced_with_input,
        min_input,
    )
}

pub fn build_success_counting_rules(
    count_success: Option<String>,
    count_failure: Option<String>,
    even: Option<String>,
    odd: Option<String>,
    deduct_failure: Option<u32>,
    subtract_failures: Option<String>,
    margin_of_success: Option<u32>,
) -> SuccessCountingRules {
    let count_success_or_failure_options = [
        count_success.is_some(),
        count_failure.is_some(),
        subtract_failures.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();
    assert!(
        count_success_or_failure_options <= 1,
        "Only one of count_success, subtract_failures or count_failure can be used"
    );

    let count_success_input: Option<Operator>;
    let count_failure_input: Option<Operator>;
    let subtract_failures_input: bool;

    let count_success_rule = count_success
        .map(|value| ("count_success", value))
        .or_else(|| count_failure.map(|value| ("count_failure", value)))
        .or_else(|| subtract_failures.map(|value| ("subtract_failure", value)))
        .or_else(|| None);

    if let Some((rule, value)) = count_success_rule {
        if rule == "count_success" {
            count_success_input = parse_operator(&value);
            count_failure_input = None;
            subtract_failures_input = false;
        } else if rule == "count_failure" {
            count_success_input = None;
            count_failure_input = parse_operator(&value);
            subtract_failures_input = false;
        } else {
            count_success_input = None;
            count_failure_input = parse_operator(&value);
            subtract_failures_input = true;
        }
    } else {
        count_success_input = None;
        count_failure_input = None;
        subtract_failures_input = false;
    }

    let even_input = match even {
        None => false,
        Some(value) => yn_tf_to_bool(value),
    };

    let odd_input = match odd {
        None => false,
        Some(value) => yn_tf_to_bool(value),
    };

    SuccessCountingRules::new(
        count_success_input,
        count_failure_input,
        even_input,
        odd_input,
        deduct_failure,
        subtract_failures_input,
        margin_of_success.unwrap_or(0u32),
    )
}
