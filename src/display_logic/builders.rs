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
        None => {
            re_roll_number = 0;
            None
        }
        Some(input) => {
            re_roll_number = parse_number(&input);
            parse_operator(&input)
        }
    };

    let re_roll_recursively_input = yn_tf_to_bool(re_roll_recursively);

    let explode_number: u32;
    let explode_input = match explode {
        None => {
            explode_number = 0;
            None
        }
        Some(input) => {
            explode_number = parse_number(&input);
            parse_operator(&input)
        }
    };

    let explode_once_input = yn_tf_to_bool(explode_once);

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

    let even_input = yn_tf_to_bool(even);

    let odd_input = yn_tf_to_bool(odd);

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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Operator;

    // --- parse_dice tests ---

    #[test]
    fn test_parse_dice_simple() {
        assert_eq!(parse_dice("1d6"), (1, 6));
        assert_eq!(parse_dice("10d20"), (10, 20));
    }

    #[test]
    fn test_parse_dice_large_numbers() {
        assert_eq!(parse_dice("100d1000"), (100, 1000));
    }

    #[test]
    #[should_panic] // Regex unwrap will panic on non-matching strings if not handled gracefully
    fn test_parse_dice_invalid_format() {
        // The regex expects digits 'd' digits.
        parse_dice("1d");
    }

    #[test]
    #[should_panic]
    fn test_parse_dice_no_d() {
        parse_dice("6");
    }

    // --- build_dice_roll_commands tests ---

    #[test]
    fn test_build_dice_roll_commands_single_group_no_mods() {
        let (commands, modifier) = build_dice_roll_commands(
            "1d6".to_string(),
            None, None, None, None
        );
        assert_eq!(modifier, 0);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].dice_count, 1);
        assert_eq!(commands[0].dice_size, 6);
        assert_eq!(commands[0].sign, 1);
        assert_eq!(commands[0].group, 1);
    }

    #[test]
    fn test_build_dice_roll_commands_single_group_with_modifier() {
        let (commands, modifier) = build_dice_roll_commands(
            "2d10+5".to_string(),
            None, None, None, None
        );
        assert_eq!(modifier, 5);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].dice_count, 2);
        assert_eq!(commands[0].dice_size, 10);
    }

    #[test]
    fn test_build_dice_roll_commands_negative_group_and_mod() {
        // "-2d6-3"
        let (commands, modifier) = build_dice_roll_commands(
            "-2d6-3".to_string(),
            None, None, None, None
        );
        assert_eq!(modifier, -3);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].sign, -1);
        assert_eq!(commands[0].dice_count, 2);
    }

    #[test]
    fn test_build_dice_roll_commands_multiple_groups() {
        // "1d6 + 2d4"
        let (commands, modifier) = build_dice_roll_commands(
            "1d6+2d4".to_string(),
            None, None, None, None
        );
        assert_eq!(modifier, 0);
        assert_eq!(commands.len(), 2);

        // Group 1
        assert_eq!(commands[0].dice_count, 1);
        assert_eq!(commands[0].dice_size, 6);
        assert_eq!(commands[0].group, 1);

        // Group 2
        assert_eq!(commands[1].dice_count, 2);
        assert_eq!(commands[1].dice_size, 4);
        assert_eq!(commands[1].group, 2);
    }

    #[test]
    fn test_build_dice_roll_commands_options_propagation() {
        let (commands, _) = build_dice_roll_commands(
            "1d20".to_string(),
            Some("lt5".to_string()),  // re_roll
            Some("y".to_string()),    // re_roll_recursively
            Some("eq20".to_string()), // explode
            Some("n".to_string()),    // explode_once
        );

        let cmd = &commands[0];
        match cmd.re_roll {
            Some(Operator::Lt(5)) => {},
            _ => panic!("Expected Lt(5)"),
        }
        assert_eq!(cmd.re_roll_recursively, true);

        match cmd.explode {
            Some(Operator::Eq(20)) => {},
            _ => panic!("Expected Eq(20)"),
        }
        assert_eq!(cmd.explode_once, false);
    }

    #[test]
    #[should_panic(expected = "re-roll number exceeds maximum dice size")]
    fn test_panic_reroll_exceeds_sides() {
        build_dice_roll_commands(
            "1d6".to_string(),
            Some("gt7".to_string()),
            None, None, None
        );
    }

    #[test]
    #[should_panic(expected = "explode number exceeds maximum dice size")]
    fn test_panic_explode_exceeds_sides() {
        build_dice_roll_commands(
            "1d6".to_string(),
            None, None,
            Some("eq7".to_string()),
            None
        );
    }

    // --- build_result_keeping_rules tests ---

    #[test]
    fn test_build_result_keeping_rules_keep_high() {
        let rules = build_result_keeping_rules(Some(3), None, None, None, None, None);
        assert_eq!(rules.keep, true);
        assert_eq!(rules.high, true);
        assert_eq!(rules.keep_or_drop_count, 3);
    }

    #[test]
    fn test_build_result_keeping_rules_keep_low() {
        let rules = build_result_keeping_rules(None, Some(2), None, None, None, None);
        assert_eq!(rules.keep, true);
        assert_eq!(rules.high, false); // keep low
        assert_eq!(rules.keep_or_drop_count, 2);
    }

    #[test]
    fn test_build_result_keeping_rules_drop_high() {
        let rules = build_result_keeping_rules(None, None, Some(1), None, None, None);
        assert_eq!(rules.keep, false);
        assert_eq!(rules.high, true); // drop high
        assert_eq!(rules.keep_or_drop_count, 1);
    }

    #[test]
    fn test_build_result_keeping_rules_drop_low() {
        let rules = build_result_keeping_rules(None, None, None, Some(4), None, None);
        assert_eq!(rules.keep, false);
        assert_eq!(rules.high, false); // drop low
        assert_eq!(rules.keep_or_drop_count, 4);
    }

    #[test]
    fn test_build_result_keeping_rules_min() {
        let rules = build_result_keeping_rules(None, None, None, None, None, Some(2));
        assert_eq!(rules.min, true);
        assert_eq!(rules.be_replaced_with, Some(2));
    }

    #[test]
    fn test_build_result_keeping_rules_max() {
        let rules = build_result_keeping_rules(None, None, None, None, Some(10), None);
        assert_eq!(rules.min, false);
        assert_eq!(rules.be_replaced_with, Some(10));
    }

    #[test]
    #[should_panic(expected = "Only one of max or min can be used")]
    fn test_build_result_keeping_rules_min_max_conflict() {
        build_result_keeping_rules(None, None, None, None, Some(10), Some(5));
    }

    #[test]
    #[should_panic(expected = "Only one of keep_high, keep_low, drop_high, drop_low can be used")]
    fn test_build_result_keeping_rules_keep_drop_conflict() {
        build_result_keeping_rules(Some(1), Some(1), None, None, None, None);
    }

    // --- build_success_counting_rules tests ---

    #[test]
    fn test_build_success_counting_rules_count_success() {
        let rules = build_success_counting_rules(
            Some("gt10".to_string()), None, None, None, None, None, None
        );
        match rules.count_success {
            Some(Operator::Gt(10)) => {},
            _ => panic!("Expected Gt(10)"),
        }
        assert!(rules.count_failure.is_none());
    }

    #[test]
    fn test_build_success_counting_rules_count_failure() {
        let rules = build_success_counting_rules(
            None, Some("lte1".to_string()), None, None, None, None, None
        );
        match rules.count_failure {
            Some(Operator::Lte(1)) => {},
            _ => panic!("Expected Lte(1)"),
        }
        assert!(rules.count_success.is_none());
    }

    #[test]
    fn test_build_success_counting_rules_subtract_failures() {
        let rules = build_success_counting_rules(
            None, None, None, None, None,
            Some("lt2".to_string()),
            None
        );
        match rules.count_failure {
            Some(Operator::Lt(2)) => {},
            _ => panic!("Expected Lt(2)"),
        }
        assert_eq!(rules.subtract_failure, true);
        assert!(rules.count_success.is_none());
    }

    #[test]
    fn test_build_success_counting_rules_even_odd() {
        let rules = build_success_counting_rules(
            None, None,
            Some("y".to_string()), // even
            Some("y".to_string()), // odd
            None, None, None
        );
        assert_eq!(rules.count_even, true);
        assert_eq!(rules.count_odd, true);
    }

    #[test]
    fn test_build_success_counting_rules_margin_deduct() {
        let rules = build_success_counting_rules(
            Some("gt5".to_string()),
            None, None, None,
            Some(2), // deduct_failure
            None,
            Some(4)  // margin_of_success
        );
        assert_eq!(rules.deduct_failure, Some(2));
        assert_eq!(rules.margin_of_success, 4);
    }

    #[test]
    #[should_panic(expected = "Only one of count_success, subtract_failures or count_failure can be used")]
    fn test_build_success_counting_rules_conflict() {
        build_success_counting_rules(
            Some("gt10".to_string()),
            Some("lt1".to_string()),
            None, None, None, None, None
        );
    }
}