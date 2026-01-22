use crate::dice_rolling_logic::success_counting_rules::SuccessCountingRulesApplied;
use crate::utils::{AbridgedTableDisplay, VerboseTableDisplay};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SuccessCountingAfterResultKeeping {
    pub rolls: Vec<SuccessCountingRulesApplied>,
    pub deductions_from_failure: u32,
    pub subtractions_from_failure: u32,
    pub margin_of_success: u32,
    pub initial_modifier: i32,
    pub final_modifier: i32,
    pub grouped_subtotals: HashMap<i32, i32>,
    pub total_before_modifier: i32,
    pub total: i32,
    pub doubled: i32,
    pub halved: f32,
    pub(crate) successes: u32,
    pub failures: u32,
    pub evens: u32,
    pub odds: u32,
}

impl SuccessCountingAfterResultKeeping {
    pub fn new(
        rolls: Vec<SuccessCountingRulesApplied>,
        deductions_from_failure: u32,
        subtractions_from_failure: u32,
        margin_of_success: u32,
        initial_modifier: i32,
        successes: u32,
        failures: u32,
        evens: u32,
        odds: u32,
    ) -> Self {
        let kept_successes = rolls
            .iter()
            .filter(|x| x.kept)
            .filter(|x| {
                (x.success == Some(true) || x.failure == Some(false))
                    || (x.success.is_none() && x.failure.is_none())
            })
            .collect::<Vec<&SuccessCountingRulesApplied>>();

        let mut grouped_subtotals: HashMap<i32, i32> = HashMap::new();

        for successes in kept_successes {
            let group_subtotal = *grouped_subtotals.entry(successes.group).or_insert(0);
            let to_add = successes.subtotal * successes.sign;
            if group_subtotal.eq(&0) {
                grouped_subtotals.insert(successes.group, to_add);
            } else {
                grouped_subtotals.insert(successes.group, group_subtotal + to_add);
            }
        }

        let final_modifier = initial_modifier
            - deductions_from_failure as i32
            - subtractions_from_failure as i32
            - margin_of_success as i32;

        let total_before_modifier = grouped_subtotals.values().sum::<i32>();
        let total = total_before_modifier + final_modifier;

        Self {
            rolls,
            deductions_from_failure,
            subtractions_from_failure,
            margin_of_success,
            initial_modifier,
            final_modifier,
            grouped_subtotals,
            total_before_modifier,
            total,
            doubled: total * 2,
            halved: total as f32 / 2.0,
            successes,
            failures,
            evens,
            odds,
        }
    }
}

fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
}

impl VerboseTableDisplay for SuccessCountingAfterResultKeeping {
    fn verbose_display(mut self) {
        self.rolls.sort_by(|a, b| a.group.cmp(&b.group));
        self.rolls.iter().for_each(|x| x.clone().verbose_display());
        let mut header = vec![
            Cell::new("Total Before Modifier"),
            Cell::new("Total"),
            Cell::new("Doubled"),
            Cell::new("Halved"),
        ];
        let mut row = vec![
            Cell::new(self.total_before_modifier),
            Cell::new(self.total),
            Cell::new(self.doubled),
            Cell::new(self.halved),
        ];

        if self.initial_modifier != self.final_modifier {
            header.push(Cell::new("Initial Modifier"));
            row.push(Cell::new(format_modifier(self.initial_modifier)));
            header.push(Cell::new("Final Modifier"));
            row.push(Cell::new(format_modifier(self.final_modifier)));
        } else {
            header.push(Cell::new("Modifier"));
            row.push(Cell::new(format_modifier(self.final_modifier)));
        }
        if self.deductions_from_failure > 0 {
            header.push(Cell::new("Deductions From Failure"));
            row.push(Cell::new(self.deductions_from_failure));
        }
        if self.subtractions_from_failure > 0 {
            header.push(Cell::new("Subtractions From Failure"));
            row.push(Cell::new(self.subtractions_from_failure));
        }
        if self.margin_of_success > 0 {
            header.push(Cell::new("Margin of Success"));
            row.push(Cell::new(self.margin_of_success));
        }
        if self.successes > 0 {
            header.push(Cell::new("Successes"));
            row.push(Cell::new(self.successes));
        }
        if self.failures > 0 {
            header.push(Cell::new("Failures"));
            row.push(Cell::new(self.failures));
        }
        if self.evens > 0 {
            header.push(Cell::new("Evens"));
            row.push(Cell::new(self.evens));
        }
        if self.odds > 0 {
            header.push(Cell::new("Odds"));
            row.push(Cell::new(self.odds));
        }

        let mut main_result = Table::new();
        main_result
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(160)
            .set_header(header)
            .add_row(row);
        println!("{main_result}");

        let mut keys: Vec<_> = self.grouped_subtotals.keys().collect();
        if keys.len() > 1 {
            keys.sort();
            let mut group_headers: Vec<String> = vec![];
            let mut groups: Vec<i32> = vec![];
            for key in &keys {
                if let Some(value) = self.grouped_subtotals.get(*key) {
                    group_headers.push(format!("dice group {}", key));
                    groups.push(*value)
                }
            }
            let mut group_subtotals = Table::new();
            group_subtotals
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_width(160)
                .set_header(group_headers)
                .add_row(groups);
            println!("{group_subtotals}");
        }
    }
}

impl AbridgedTableDisplay for SuccessCountingAfterResultKeeping {
    fn abridged_display(mut self) {
        self.rolls.sort_by(|a, b| a.group.cmp(&b.group));
        let mut current_group = 1;
        for i in 0..self.rolls.len() {
            let curr = &self.rolls[i];
            if curr.group != current_group {
                current_group = curr.group;
                println!()
            }
            if curr.kept
                && ((curr.success == Some(true) || curr.failure == Some(false))
                    || (curr.success.is_none() && curr.failure.is_none()))
            {
                print!("{}/{}, ", curr.final_roll, curr.dice_size)
            }
        }
        println!();
        println!(
            "Modifier: {}, Total: {}",
            format_modifier(self.final_modifier),
            self.total
        );
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_applied_roll(
        group: i32,
        sign: i32,
        roll_number: u32,
        final_roll: u32,
        kept: bool,
        success: Option<bool>,
        failure: Option<bool>,
    ) -> SuccessCountingRulesApplied {
        SuccessCountingRulesApplied::new(
            group,
            sign,
            roll_number,
            6, // dice_size
            final_roll,
            vec![],
            vec![],
            final_roll as i32, // subtotal = final_roll (no explosions)
            kept,
            None, // replaced_roll
            success,
            failure,
            false, // subtracted
            0,     // deductions
        )
    }

    #[test]
    fn test_basic_grouped_subtotals_and_total() {
        let rolls = vec![
            mock_applied_roll(1, 1, 1, 4, true, None, None),
            mock_applied_roll(1, 1, 2, 5, true, None, None),
            mock_applied_roll(2, 1, 3, 3, true, None, None),
        ];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            0, // deductions
            0, // subtractions
            0, // margin
            2, // initial modifier
            0, 0, 0, 0, // successes, etc.
        );

        let expected_grouped = HashMap::from([(1, 9), (2, 3)]);
        assert_eq!(result.grouped_subtotals, expected_grouped);
        assert_eq!(result.total_before_modifier, 12);
        assert_eq!(result.final_modifier, 2); // 2 - 0 - 0 - 0
        assert_eq!(result.total, 14);
        assert_eq!(result.doubled, 28);
        assert_eq!(result.halved, 7.0);
    }

    #[test]
    fn test_negative_sign_group() {
        let rolls = vec![
            mock_applied_roll(1, 1, 1, 6, true, None, None),   // +6
            mock_applied_roll(2, -1, 2, 4, true, None, None),  // -4
        ];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            0, 0, 0, 0,
            0, 0, 0, 0,
        );

        let expected_grouped = HashMap::from([(1, 6), (2, -4)]);
        assert_eq!(result.grouped_subtotals, expected_grouped);
        assert_eq!(result.total_before_modifier, 2);
        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_neutral_rolls_included() {
        let rolls = vec![
            mock_applied_roll(1, 1, 1, 4, true, None, None), // neutral → included
        ];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            0, 0, 0, 0,
            0, 0, 0, 0,
        );

        assert_eq!(result.total_before_modifier, 4);
    }

    #[test]
    fn test_only_successes_included_when_success_rules_active() {
        let rolls = vec![
            mock_applied_roll(1, 1, 1, 6, true, Some(true), None),   // success → included
            mock_applied_roll(1, 1, 2, 3, true, Some(false), None),  // failure → excluded
            mock_applied_roll(1, 1, 3, 4, true, None, None),        // neutral → included? But shouldn't happen with rules...
        ];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            0, 0, 0, 0,
            1, 1, 0, 0, // successes=1, failures=1 (but not used in filtering)
        );

        // Only roll 1 and 3 are included? But in real usage, roll 3 wouldn't be neutral.
        // However, per your filter:
        // - roll1: success=true → included
        // - roll2: success=false → NOT included
        // - roll3: neutral → included
        assert_eq!(result.total_before_modifier, 6 + 4); // 10
    }

    #[test]
    fn test_subtotal_excludes_failed_rolls_under_success_counting() {
        // Realistic: all rolls have success flag set
        let rolls = vec![
            mock_applied_roll(1, 1, 1, 6, true, Some(true), None),   // success
            mock_applied_roll(1, 1, 2, 3, true, Some(false), None),  // failure → excluded
            mock_applied_roll(1, 1, 3, 2, true, Some(false), None),  // failure → excluded
        ];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            0, 0, 0, 0,
            1, 2, 0, 0,
        );

        // Only the success (6) is included
        assert_eq!(result.total_before_modifier, 6);
    }

    #[test]
    fn test_final_modifier_calculation() {
        let rolls = vec![mock_applied_roll(1, 1, 1, 5, true, None, None)];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            2, // deductions
            3, // subtractions
            1, // margin_of_success
            10, // initial modifier
            0, 0, 0, 0,
        );

        // final_modifier = 10 - 2 - 3 - 1 = 4
        assert_eq!(result.initial_modifier, 10);
        assert_eq!(result.final_modifier, 4);
        assert_eq!(result.total_before_modifier, 5);
        assert_eq!(result.total, 9);
    }

    #[test]
    fn test_empty_rolls() {
        let rolls = vec![];

        let result = SuccessCountingAfterResultKeeping::new(
            rolls,
            0, 0, 0, 5,
            0, 0, 0, 0,
        );

        assert!(result.grouped_subtotals.is_empty());
        assert_eq!(result.total_before_modifier, 0);
        assert_eq!(result.total, 5); // 0 + 5
        assert_eq!(result.doubled, 10);
        assert_eq!(result.halved, 2.5);
    }
}