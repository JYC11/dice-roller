use crate::dice_rolling_logic::result_keeping_rules::ResultKeepingRulesApplied;
use crate::dice_rolling_logic::roll_result::SuccessCountingAfterResultKeeping;
use crate::enums::Operator;
use crate::utils::{apply_operator, VerboseTableDisplay};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};

#[derive(Copy, Clone)]
pub struct SuccessCountingRules {
    pub count_success: Option<Operator>,
    pub count_failure: Option<Operator>,
    pub count_even: bool,
    pub count_odd: bool,
    pub deduct_failure: Option<u32>, // deduct each by failure n
    pub subtract_failure: bool,      // deduct the entire dice roll
    pub margin_of_success: u32,
}

impl SuccessCountingRules {
    pub fn new(
        count_success: Option<Operator>,
        count_failure: Option<Operator>,
        count_even: bool,
        count_odd: bool,
        deduct_failure: Option<u32>,
        subtract_failure: bool,
        margin_of_success: u32,
    ) -> Self {
        // panic when
        // count success and count failure are both some
        // have to be either or
        assert!(!(count_success.is_some() && count_failure.is_some()));
        SuccessCountingRules {
            count_success,
            count_failure,
            count_even,
            count_odd,
            deduct_failure,
            subtract_failure,
            margin_of_success,
        }
    }

    pub fn count_successes(
        &self,
        result_keeping_rules_applied: &mut [ResultKeepingRulesApplied],
        modifier: i32,
    ) -> SuccessCountingAfterResultKeeping {
        let mut success_counting_rules_applied: Vec<SuccessCountingRulesApplied> = vec![];
        let mut evens = 0;
        let mut odds = 0;
        let mut total_subtracted = 0;
        let mut total_deducted = 0;
        let mut successes = 0;
        let mut failures = 0;

        for roll in result_keeping_rules_applied {
            let success = self.check_success(roll.final_roll);
            let failure = self.check_failure(roll.final_roll);
            let (subtracted, deduction) =
                self.calculate_deductions(roll.final_roll, success, failure);

            if roll.kept {
                if let Some(true) = success {
                    successes += 1;
                } else if let Some(false) = success {
                    failures += 1;
                }

                if let Some(true) = failure {
                    failures += 1;
                } else if let Some(false) = failure {
                    successes += 1;
                }

                total_subtracted += subtracted;
                total_deducted += deduction;

                // Count even and odd rolls
                if self.count_even && roll.final_roll % 2 == 0 {
                    evens += 1;
                }
                if self.count_odd && roll.final_roll % 2 != 0 {
                    odds += 1;
                }
            }

            success_counting_rules_applied.push(SuccessCountingRulesApplied::new(
                roll.group,
                roll.sign,
                roll.roll_number,
                roll.dice_size,
                roll.final_roll,
                roll.discarded_rolls.clone(),
                roll.exploded_rolls.clone(),
                roll.subtotal,
                roll.kept,
                roll.replaced_roll,
                success,
                failure,
                subtracted > 0,
                deduction,
            ));
        }

        success_counting_rules_applied.sort_by(|a, b| a.roll_number.cmp(&b.roll_number));

        SuccessCountingAfterResultKeeping::new(
            success_counting_rules_applied,
            total_deducted,
            total_subtracted,
            self.margin_of_success,
            modifier,
            successes,
            failures,
            evens,
            odds,
        )
    }

    fn check_success(&self, roll_value: u32) -> Option<bool> {
        self.count_success
            .map(|operator| apply_operator(operator, &roll_value))
    }

    fn check_failure(&self, roll_value: u32) -> Option<bool> {
        self.count_failure
            .map(|operator| apply_operator(operator, &roll_value))
    }

    fn calculate_deductions(
        &self,
        roll_value: u32,
        success: Option<bool>,
        failure: Option<bool>,
    ) -> (u32, u32) {
        let mut total_subtracted = 0;
        let mut total_deducted = 0;

        if self.subtract_failure {
            if matches!(success, Some(false)) || matches!(failure, Some(true)) {
                total_subtracted += roll_value;
            }
        }

        if let Some(value) = self.deduct_failure {
            if matches!(success, Some(false)) || matches!(failure, Some(true)) {
                total_deducted += value;
            }
        }

        (total_subtracted, total_deducted)
    }
}

#[derive(Clone)]
pub struct SuccessCountingRulesApplied {
    pub group: i32,
    pub sign: i32,
    pub roll_number: u32,
    pub dice_size: u32,
    pub final_roll: u32,
    pub discarded_rolls: Vec<u32>,
    pub exploded_rolls: Vec<u32>,
    pub subtotal: i32,
    pub kept: bool,
    pub replaced_roll: Option<u32>,
    pub success: Option<bool>,
    pub failure: Option<bool>,
    pub subtracted: bool,
    pub deductions: u32,
}

impl SuccessCountingRulesApplied {
    pub fn new(
        group: i32,
        sign: i32,
        roll_number: u32,
        dice_size: u32,
        final_roll: u32,
        discarded_rolls: Vec<u32>,
        exploded_rolls: Vec<u32>,
        subtotal: i32,
        kept: bool,
        replaced_roll: Option<u32>,
        success: Option<bool>,
        failure: Option<bool>,
        subtracted: bool,
        deductions: u32,
    ) -> Self {
        Self {
            group,
            sign,
            roll_number,
            dice_size,
            final_roll,
            discarded_rolls,
            exploded_rolls,
            subtotal,
            kept,
            replaced_roll,
            success,
            failure,
            subtracted,
            deductions,
        }
    }
}

impl VerboseTableDisplay for SuccessCountingRulesApplied {
    fn verbose_display(self) {
        let mut table1 = Table::new();

        let added = self.sign > 0;

        let mut header = vec![
            Cell::new("Dice group"),
            Cell::new("Dice"),
            Cell::new("Added"),
            Cell::new("Roll Number"),
            Cell::new("Final roll"),
        ];

        let mut row = vec![
            Cell::new(self.group),
            Cell::new(self.dice_size),
            Cell::new(added),
            Cell::new(self.roll_number),
            Cell::new(self.final_roll),
        ];

        if !self.discarded_rolls.is_empty() {
            header.push(Cell::new("Discarded rolls from re-rolling"));
            row.push(Cell::new(format!("{:?}", self.discarded_rolls)));
        }

        if !self.exploded_rolls.is_empty() {
            header.push(Cell::new("Exploded rolls"));
            row.push(Cell::new(format!("{:?}", self.exploded_rolls)));
        }

        if let Some(target) = &self.replaced_roll {
            header.push(Cell::new("Replaced"));
            row.push(Cell::new(format!(
                "{} has been replaced with {}",
                target, self.final_roll
            )));
        }

        if !self.kept {
            header.push(Cell::new("Kept"));
            row.push(Cell::new(self.kept));
        }

        let success: Option<&str>;
        match (self.success, self.failure) {
            (Some(true), Some(false)) => success = Some("true"),
            (Some(false), Some(true)) => success = Some("false"),
            (_, _) => success = None,
        }

        if let Some(target) = success {
            header.push(Cell::new("Success"));
            row.push(Cell::new(target));
        }

        if self.deductions > 0 {
            header.push(Cell::new("Deductions from final roll"));
            row.push(Cell::new(self.deductions));
        }

        if self.subtracted {
            header.push(Cell::new("Subtracted from final roll"));
            row.push(Cell::new(self.subtracted));
        }

        header.push(Cell::new("Subtotal"));
        row.push(Cell::new(self.subtotal));

        table1
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(160)
            .set_header(header)
            .add_row(row);
        println!("{table1}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice_rolling_logic::result_keeping_rules::ResultKeepingRulesApplied;

    fn mock_kept_roll(
        roll_number: u32,
        final_roll: u32,
        kept: bool,
    ) -> ResultKeepingRulesApplied {
        ResultKeepingRulesApplied::new(
            1,               // group
            1,               // sign
            roll_number,
            6,               // dice_size
            final_roll,
            vec![],          // discarded
            vec![],          // exploded
            final_roll as i32, // subtotal
            kept,
            None,            // replaced_roll
        )
    }

    #[test]
    fn test_count_success_gte_5() {
        let mut rolls = vec![
            mock_kept_roll(1, 6, true),
            mock_kept_roll(2, 4, true),
            mock_kept_roll(3, 5, true),
            mock_kept_roll(4, 3, false), // not kept → ignored
        ];

        let rules = SuccessCountingRules::new(
            Some(Operator::Gte(5)), // success if >=5
            None,
            false,
            false,
            None,
            false,
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.successes, 2); // 6 and 5
        assert_eq!(result.failures, 1);  // 4 is <5 → failure
        assert_eq!(result.evens, 0);
        assert_eq!(result.odds, 0);
    }

    #[test]
    fn test_count_failure_lte_2() {
        let mut rolls = vec![
            mock_kept_roll(1, 1, true),
            mock_kept_roll(2, 3, true),
            mock_kept_roll(3, 2, true),
            mock_kept_roll(4, 6, false), // not kept
        ];

        let rules = SuccessCountingRules::new(
            None,
            Some(Operator::Lte(2)), // failure if <=2
            false,
            false,
            None,
            false,
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.failures, 2); // 1 and 2
        assert_eq!(result.successes, 1); // 3 is >2 → success
    }

    #[test]
    fn test_count_even_and_odd() {
        let mut rolls = vec![
            mock_kept_roll(1, 2, true), // even
            mock_kept_roll(2, 3, true), // odd
            mock_kept_roll(3, 4, true), // even
            mock_kept_roll(4, 5, false), // not kept
        ];

        let rules = SuccessCountingRules::new(
            None,
            None,
            true,  // count even
            true,  // count odd
            None,
            false,
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.evens, 2); // 2,4
        assert_eq!(result.odds, 1);  // 3
        assert_eq!(result.successes, 0);
        assert_eq!(result.failures, 0);
    }

    #[test]
    fn test_deduct_per_failure() {
        let mut rolls = vec![
            mock_kept_roll(1, 1, true), // failure (if success = >=5)
            mock_kept_roll(2, 6, true), // success
            mock_kept_roll(3, 3, true), // failure
        ];

        let rules = SuccessCountingRules::new(
            Some(Operator::Gte(5)),
            None,
            false,
            false,
            Some(2), // deduct 2 per failure
            false,
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.successes, 1);
        assert_eq!(result.failures, 2);
        assert_eq!(result.deductions_from_failure, 4); // 2 failures × 2
        assert_eq!(result.subtractions_from_failure, 0);
    }

    #[test]
    fn test_subtract_entire_roll_on_failure() {
        let mut rolls = vec![
            mock_kept_roll(1, 1, true), // failure → subtract 1
            mock_kept_roll(2, 6, true), // success → no subtract
            mock_kept_roll(3, 3, true), // failure → subtract 3
        ];

        let rules = SuccessCountingRules::new(
            Some(Operator::Gte(5)),
            None,
            false,
            false,
            None,
            true, // subtract entire roll on failure
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.subtractions_from_failure, 1 + 3); // 4
        assert_eq!(result.deductions_from_failure, 0);
    }

    #[test]
    fn test_only_kept_rolls_counted() {
        let mut rolls = vec![
            mock_kept_roll(1, 6, false), // not kept → should not count
            mock_kept_roll(2, 2, true),  // kept, but failure
        ];

        let rules = SuccessCountingRules::new(
            Some(Operator::Gte(5)),
            None,
            false,
            false,
            None,
            false,
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.successes, 0);
        assert_eq!(result.failures, 1); // only the kept 2
    }

    #[test]
    fn test_no_rules_active() {
        let mut rolls = vec![
            mock_kept_roll(1, 4, true),
            mock_kept_roll(2, 5, true),
        ];

        let rules = SuccessCountingRules::new(
            None,
            None,
            false,
            false,
            None,
            false,
            0,
        );

        let result = rules.count_successes(&mut rolls, 0);
        assert_eq!(result.successes, 0);
        assert_eq!(result.failures, 0);
        assert_eq!(result.evens, 0);
        assert_eq!(result.odds, 0);
        assert_eq!(result.deductions_from_failure, 0);
        assert_eq!(result.subtractions_from_failure, 0);
    }

    #[test]
    fn test_margin_and_modifier_passed_through() {
        let mut rolls = vec![mock_kept_roll(1, 6, true)];

        let rules = SuccessCountingRules::new(
            Some(Operator::Gte(5)),
            None,
            false,
            false,
            None,
            false,
            10, // margin_of_success = 10
        );

        let result = rules.count_successes(&mut rolls, -2); // modifier = -2
        assert_eq!(result.margin_of_success, 10);
        assert_eq!(result.initial_modifier, -2);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_cannot_have_both_success_and_failure_rules() {
        SuccessCountingRules::new(
            Some(Operator::Gte(5)),
            Some(Operator::Lte(2)),
            false,
            false,
            None,
            false,
            0,
        );
    }
}