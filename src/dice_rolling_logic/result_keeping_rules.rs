use crate::dice_rolling_logic::roll_command::InitialDiceRollResult;

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
        dice_rolls: &mut [InitialDiceRollResult],
    ) -> Vec<ResultKeepingRulesApplied> {
        if self.high {
            // Sort descending by final_roll for easier access to highest
            dice_rolls.sort_by(|a, b| b.final_roll.cmp(&a.final_roll));
        } else {
            // Sort ascending by final_roll for easier access to lowest
            dice_rolls.sort_by(|a, b| a.final_roll.cmp(&b.final_roll));
        }

        dice_rolls
            .iter()
            .enumerate()
            .map(|(i, roll)| {
                let should_keep = self.should_keep_roll(i);
                let replacement_roll = self.replacement_roll(roll.final_roll);

                let final_roll = replacement_roll.unwrap_or(roll.final_roll);
                let subtotal = roll.subtotal - roll.final_roll as i32 + final_roll as i32;
                let replaced_roll = replacement_roll.is_some().then(|| roll.final_roll);

                ResultKeepingRulesApplied::new(
                    roll.group,
                    roll.sign,
                    roll.roll_number,
                    roll.dice_size,
                    final_roll,
                    roll.discarded_rolls.clone(),
                    roll.exploded_rolls.clone(),
                    subtotal,
                    should_keep,
                    replaced_roll,
                )
            })
            .collect()
    }

    fn should_keep_roll(&self, index: usize) -> bool {
        if self.keep_or_drop_count == 0 {
            return true; // Default to keeping if no specific count is set
        }

        let within_limit = index < self.keep_or_drop_count as usize;
        match self.keep {
            true => within_limit,
            false => !within_limit,
        }
    }

    fn replacement_roll(&self, roll_value: u32) -> Option<u32> {
        if let Some(replacement) = self.be_replaced_with {
            if self.min && roll_value < replacement || !self.min && roll_value > replacement {
                return Some(replacement);
            }
        }
        None
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
