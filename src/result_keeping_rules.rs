use crate::roll_command::InitialDiceRollResult;

#[derive(Copy, Clone)]
pub struct ResultKeepingRules {
    keep: bool,
    high: bool,
    keep_or_drop_count: u32,
    be_replaced_with: Option<u32>,
    min: bool,
}

// keep = true, high = true -> keep the highest n roll(s)
// keep = false, high = true -> drop the highest n roll(s)
// keep = false, high = false -> drop the lowest n roll(s)
// keep = true, high = false -> keep the lowest n roll(s)

impl ResultKeepingRules {
    pub fn new(
        keep: bool,
        high: bool,
        keep_or_drop_count: u32,
        be_replaced_with: Option<u32>,
        min: bool,
    ) -> Self {
        Self {
            keep,
            high,
            keep_or_drop_count,
            be_replaced_with,
            min,
        }
    }

    pub fn process_results(
        &self,
        individual_dice_roll_results: &mut [InitialDiceRollResult],
    ) -> Vec<ResultKeepingRulesApplied> {
        let mut processed_results: Vec<ResultKeepingRulesApplied> = vec![];
        let size = individual_dice_roll_results.len();

        individual_dice_roll_results.sort_by(|a, b| b.final_roll.cmp(&a.final_roll));
        // will be sorted descending byt final_roll, biggest first

        // size = 6
        // idx = 0, 1, 2, 3, 4, 5
        // formula = size - 1 - idx
        // revs = 5, 4, 3, 2, 1, 0

        for i in 0..size {
            let mut idx: usize = i;
            let mut should_keep = true;

            if self.keep_or_drop_count > 0 {
                if !self.high {
                    idx = size - 1 - i;
                    // reverses the index to iterate from the back when selecting lowest rolls
                }
                let within_limit = i < self.keep_or_drop_count as usize;
                if self.keep && within_limit {
                    // keep highest
                    should_keep = true;
                } else if self.keep && !within_limit {
                    should_keep = false;
                }
                if !self.keep && within_limit {
                    // drop highest
                    should_keep = false;
                } else if !self.keep && !within_limit {
                    should_keep = true;
                }
            }

            let mut to_be_replaced_with: Option<u32> = None;
            match self.be_replaced_with {
                None => {}
                Some(be_replaced_with) => {
                    if self.min {
                        if individual_dice_roll_results[idx].final_roll < be_replaced_with {
                            to_be_replaced_with = Some(be_replaced_with);
                        }
                    } else if individual_dice_roll_results[idx].final_roll > be_replaced_with {
                        to_be_replaced_with = Some(be_replaced_with);
                    }
                }
            }
            let cloned = individual_dice_roll_results[idx].clone();

            let mut new_final_roll = cloned.final_roll;
            let mut new_subtotal = cloned.subtotal;
            let mut replaced_roll = None;
            match to_be_replaced_with {
                None => {}
                Some(to_be_replaced_with) => {
                    replaced_roll = Some(cloned.final_roll);
                    new_final_roll = to_be_replaced_with;
                    new_subtotal =
                        cloned.subtotal - cloned.final_roll as i32 + to_be_replaced_with as i32;
                }
            }

            processed_results.push(ResultKeepingRulesApplied::new(
                cloned.group,
                cloned.sign,
                cloned.roll_number,
                cloned.dice_size,
                new_final_roll,
                cloned.discarded_rolls,
                cloned.exploded_rolls,
                new_subtotal,
                should_keep,
                replaced_roll,
            ))
        }
        // will be sorted descending by roll_number, biggest last

        processed_results
    }
}

#[derive(Clone)]
pub struct ResultKeepingRulesApplied {
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
}

impl ResultKeepingRulesApplied {
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
        }
    }
}
