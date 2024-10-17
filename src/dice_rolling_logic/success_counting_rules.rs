use crate::enums::Operator;
use crate::dice_rolling_logic::result_keeping_rules::ResultKeepingRulesApplied;
use crate::display_logic::roll_result::SuccessCountingAfterResultKeeping;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};
use crate::utils::TableDisplay;

#[derive(Copy, Clone)]
pub struct SuccessCountingRules {
    count_success: Option<Operator>,
    count_failure: Option<Operator>,
    count_even: bool,
    count_odd: bool,
    deduct_failure: Option<u32>, // deduct each by failure n
    subtract_failure: bool,      // deduct the entire dice roll
    margin_of_success: u32,
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
            let mut success = None;
            let mut failure = None;
            let mut subtracted = false;
            let mut deduction = 0;

            if roll.kept {
                success = match self.count_success {
                    None => None,
                    Some(operator) => match operator {
                        Operator::Eq(target) => Some(roll.final_roll == target),
                        Operator::Gt(target) => Some(roll.final_roll > target),
                        Operator::Gte(target) => Some(roll.final_roll >= target),
                        Operator::Lt(target) => Some(roll.final_roll < target),
                        Operator::Lte(target) => Some(roll.final_roll <= target),
                    },
                };
                match success {
                    None => {}
                    Some(val) => {
                        if val {
                            successes += 1
                        } else {
                            failures += 1
                        }
                    }
                }

                failure = match self.count_failure {
                    None => None,
                    Some(operator) => match operator {
                        Operator::Eq(target) => Some(roll.final_roll == target),
                        Operator::Gt(target) => Some(roll.final_roll > target),
                        Operator::Gte(target) => Some(roll.final_roll >= target),
                        Operator::Lt(target) => Some(roll.final_roll < target),
                        Operator::Lte(target) => Some(roll.final_roll <= target),
                    },
                };
                match failure {
                    None => {}
                    Some(val) => {
                        if !val {
                            successes += 1
                        } else {
                            failures += 1
                        }
                    }
                }

                if self.subtract_failure {
                    match success {
                        None => {}
                        Some(success) => {
                            if !success {
                                subtracted = true;
                                total_subtracted += roll.final_roll;
                            }
                        }
                    }
                    match failure {
                        None => {}
                        Some(failure) => {
                            if failure {
                                subtracted = true;
                                total_subtracted += roll.final_roll;
                            }
                        }
                    }
                }

                match self.deduct_failure {
                    None => {}
                    Some(value) => {
                        match success {
                            None => {}
                            Some(success) => {
                                if !success {
                                    deduction += value;
                                    total_deducted += value;
                                }
                            }
                        }
                        match failure {
                            None => {}
                            Some(failure) => {
                                if failure {
                                    deduction += value;
                                    total_deducted += value
                                }
                            }
                        }
                    }
                }

                if self.count_even && roll.final_roll % 2 == 0 {
                    evens += 1
                }
                if self.count_odd && roll.final_roll % 2 != 0 {
                    odds += 1
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
                subtracted,
                deduction,
            ))
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

impl TableDisplay for SuccessCountingRulesApplied {
    fn display(self) {
        let mut table1 = Table::new();

        let added = self.sign > 0;

        let mut success = None;
        if self.success == Some(true) || self.failure == Some(false) {
            success = Some("true");
        }
        if self.success == Some(false) || self.failure == Some(true) {
            success = Some("false");
        }

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

        match self.replaced_roll {
            None => {}
            Some(replaced_roll) => {
                header.push(Cell::new("Replaced"));
                row.push(Cell::new(format!(
                    "{} has been replaced with {}",
                    replaced_roll, self.final_roll
                )));
            }
        }

        if !self.kept {
            header.push(Cell::new("Kept"));
            row.push(Cell::new(self.kept));
        }

        if success.is_some() {
            header.push(Cell::new("Success"));
            row.push(Cell::new(success.unwrap_or("-")));
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
