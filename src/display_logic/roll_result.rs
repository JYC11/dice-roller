use crate::dice_rolling_logic::success_counting_rules::SuccessCountingRulesApplied;
use crate::utils::{AbridgedTableDisplay, VerboseTableDisplay};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SuccessCountingAfterResultKeeping {
    rolls: Vec<SuccessCountingRulesApplied>,
    deductions_from_failure: u32,
    subtractions_from_failure: u32,
    margin_of_success: u32,
    initial_modifier: i32,
    final_modifier: i32,
    grouped_subtotals: HashMap<i32, i32>,
    total_before_modifier: i32,
    total: i32,
    doubled: i32,
    halved: f32,
    successes: u32,
    failures: u32,
    evens: u32,
    odds: u32,
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
