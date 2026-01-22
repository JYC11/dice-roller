use crate::dice_rolling_logic::roll_command::InitialDiceRollResult;

#[derive(Copy, Clone)]
pub struct ResultKeepingRules {
    pub keep: bool,
    pub high: bool,
    pub keep_or_drop_count: u32,
    pub be_replaced_with: Option<u32>,
    pub min: bool,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice_rolling_logic::roll_command::InitialDiceRollResult;

    fn mock_roll(
        group: i32,
        sign: i32,
        roll_number: u32,
        dice_size: u32,
        final_roll: u32,
        discarded: Vec<u32>,
        exploded: Vec<u32>,
    ) -> InitialDiceRollResult {
        let subtotal = final_roll as i32 + exploded.iter().map(|&x| x as i32).sum::<i32>();
        InitialDiceRollResult {
            group,
            sign,
            roll_number,
            dice_size,
            final_roll,
            discarded_rolls: discarded,
            exploded_rolls: exploded,
            subtotal,
        }
    }

    #[test]
    fn test_keep_highest_2_of_4() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 2, vec![], vec![]),
            mock_roll(1, 1, 2, 6, 5, vec![], vec![]),
            mock_roll(1, 1, 3, 6, 3, vec![], vec![]),
            mock_roll(1, 1, 4, 6, 6, vec![], vec![]),
        ];

        let rules = ResultKeepingRules::new(true, true, 2, None, false);
        let results = rules.process_results(&mut rolls);

        // Should sort descending: [6,5,3,2]
        // Keep first 2 → kept: 6,5; drop: 3,2
        let kept_values: Vec<u32> = results.iter().filter(|r| r.kept).map(|r| r.final_roll).collect();
        let dropped_values: Vec<u32> = results.iter().filter(|r| !r.kept).map(|r| r.final_roll).collect();

        assert_eq!(kept_values.len(), 2);
        assert_eq!(dropped_values.len(), 2);
        assert!(kept_values.contains(&6));
        assert!(kept_values.contains(&5));
        assert!(dropped_values.contains(&3));
        assert!(dropped_values.contains(&2));
    }

    #[test]
    fn test_drop_lowest_1_of_3() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 4, vec![], vec![]),
            mock_roll(1, 1, 2, 6, 1, vec![], vec![]),
            mock_roll(1, 1, 3, 6, 6, vec![], vec![]),
        ];

        let rules = ResultKeepingRules::new(false, false, 1, None, false); // drop lowest 1
        let results = rules.process_results(&mut rolls);

        // Sorted ascending: [1,4,6] → drop index 0 → keep 4,6
        let kept: Vec<u32> = results.iter().filter(|r| r.kept).map(|r| r.final_roll).collect();
        assert_eq!(kept.len(), 2);
        assert!(kept.contains(&4));
        assert!(kept.contains(&6));
        assert_eq!(results.iter().find(|r| r.final_roll == 1).unwrap().kept, false);
    }

    #[test]
    fn test_keep_lowest_2_of_4() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 6, vec![], vec![]),
            mock_roll(1, 1, 2, 6, 2, vec![], vec![]),
            mock_roll(1, 1, 3, 6, 5, vec![], vec![]),
            mock_roll(1, 1, 4, 6, 1, vec![], vec![]),
        ];

        let rules = ResultKeepingRules::new(true, false, 2, None, false); // keep lowest 2
        let results = rules.process_results(&mut rolls);

        // Sorted ascending: [1,2,5,6] → keep first 2
        let kept: Vec<u32> = results.iter().filter(|r| r.kept).map(|r| r.final_roll).collect();
        assert_eq!(kept, vec![1, 2]); // order may vary by roll_number, but values should be present
        assert!(kept.contains(&1));
        assert!(kept.contains(&2));
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn test_drop_highest_1_of_3() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 3, vec![], vec![]),
            mock_roll(1, 1, 2, 6, 6, vec![], vec![]),
            mock_roll(1, 1, 3, 6, 4, vec![], vec![]),
        ];

        let rules = ResultKeepingRules::new(false, true, 1, None, false); // drop highest 1
        let results = rules.process_results(&mut rolls);

        // Sorted desc: [6,4,3] → drop index 0 (6)
        let kept: Vec<u32> = results.iter().filter(|r| r.kept).map(|r| r.final_roll).collect();
        assert_eq!(kept.len(), 2);
        assert!(kept.contains(&3));
        assert!(kept.contains(&4));
        assert_eq!(results.iter().find(|r| r.final_roll == 6).unwrap().kept, false);
    }

    #[test]
    fn test_keep_count_zero_keeps_all() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 2, vec![], vec![]),
            mock_roll(1, 1, 2, 6, 5, vec![], vec![]),
        ];

        let rules = ResultKeepingRules::new(true, true, 0, None, false);
        let results = rules.process_results(&mut rolls);

        assert!(results.iter().all(|r| r.kept));
    }

    #[test]
    fn test_replace_if_below_min() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 2, vec![], vec![]), // roll_number = 1
            mock_roll(1, 1, 2, 6, 4, vec![], vec![]), // roll_number = 2
        ];

        let rules = ResultKeepingRules::new(false, false, 0, Some(3), true);
        let results = rules.process_results(&mut rolls);

        let roll_with_2 = results.iter().find(|r| r.roll_number == 1).unwrap();
        let roll_with_4 = results.iter().find(|r| r.roll_number == 2).unwrap();

        assert_eq!(roll_with_2.replaced_roll, Some(2));
        assert_eq!(roll_with_2.final_roll, 3);
        assert_eq!(roll_with_2.subtotal, 3);

        assert_eq!(roll_with_4.replaced_roll, None);
        assert_eq!(roll_with_4.final_roll, 4);
        assert_eq!(roll_with_4.subtotal, 4);
    }

    #[test]
    fn test_replace_if_above_max() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 5, vec![], vec![]), // roll_number = 1
            mock_roll(1, 1, 2, 6, 3, vec![], vec![]), // roll_number = 2
        ];

        let rules = ResultKeepingRules::new(false, false, 0, Some(4), false);
        let results = rules.process_results(&mut rolls);

        // Find result by roll_number
        let roll_with_5 = results.iter().find(|r| r.roll_number == 1).unwrap();
        let roll_with_3 = results.iter().find(|r| r.roll_number == 2).unwrap();

        // 5 > 4 → replaced with 4
        assert_eq!(roll_with_5.replaced_roll, Some(5));
        assert_eq!(roll_with_5.final_roll, 4);
        assert_eq!(roll_with_5.subtotal, 4);

        // 3 <= 4 → unchanged
        assert_eq!(roll_with_3.replaced_roll, None);
        assert_eq!(roll_with_3.final_roll, 3);
        assert_eq!(roll_with_3.subtotal, 3);
    }

    #[test]
    fn test_keep_highest_2_with_replacement() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 1, vec![], vec![]), // will be replaced → becomes 2
            mock_roll(1, 1, 2, 6, 6, vec![], vec![]), // stays 6
            mock_roll(1, 1, 3, 6, 3, vec![], vec![]), // stays 3
            mock_roll(1, 1, 4, 6, 2, vec![], vec![]), // stays 2
        ];

        // Replace any roll < 2 with 2, then keep highest 2
        let rules = ResultKeepingRules::new(true, true, 2, Some(2), true);
        let results = rules.process_results(&mut rolls);

        // After replacement: [2,6,3,2] → sorted desc: [6,3,2,2] → keep 6 and 3
        let kept: Vec<u32> = results.iter().filter(|r| r.kept).map(|r| r.final_roll).collect();
        assert_eq!(kept.len(), 2);
        assert!(kept.contains(&6));
        assert!(kept.contains(&3));

        // Verify replacement happened
        let replaced_roll = results.iter().find(|r| r.replaced_roll == Some(1)).unwrap();
        assert_eq!(replaced_roll.final_roll, 2);
    }

    #[test]
    fn test_keep_more_than_available() {
        let mut rolls = vec![
            mock_roll(1, 1, 1, 6, 4, vec![], vec![]),
            mock_roll(1, 1, 2, 6, 5, vec![], vec![]),
        ];

        // Try to keep top 5 of 2 rolls → should keep both
        let rules = ResultKeepingRules::new(true, true, 5, None, false);
        let results = rules.process_results(&mut rolls);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.kept));
    }
}