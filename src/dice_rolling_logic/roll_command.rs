use crate::enums::Operator;
use rand::Rng;
use crate::utils::apply_operator;

#[derive(Copy, Clone, Debug)]
pub struct DiceRollCommand {
    pub group: i32,
    pub sign: i32,
    pub dice_count: u32,
    pub dice_size: u32,
    pub re_roll: Option<Operator>,
    pub re_roll_recursively: bool,
    pub explode: Option<Operator>,
    pub explode_once: bool,
}

impl DiceRollCommand {
    pub fn new(
        group: i32,
        sign: i32,
        dice_count: u32,
        dice_size: u32,
        re_roll: Option<Operator>,
        re_roll_recursively: bool,
        explode: Option<Operator>,
        explode_once: bool,
    ) -> Self {
        Self {
            group,
            sign,
            dice_count,
            dice_size,
            re_roll,
            re_roll_recursively,
            explode,
            explode_once,
        }
    }

    pub fn roll_dice(&self) -> Vec<InitialDiceRollResult> {
        (1..=self.dice_count)
            .map(|roll_number| self.roll_single_dice(roll_number))
            .collect()
    }

    fn roll_single_dice(&self, roll_number: u32) -> InitialDiceRollResult {
        let mut rng = rand::thread_rng();
        let mut discarded_rolls = vec![];
        let mut exploded_rolls = vec![];
        let mut roll = rng.gen_range(1..=self.dice_size);

        if let Some(target) = &self.re_roll {
            roll = self.apply_re_rolls(&mut rng, roll, target, &mut discarded_rolls);
        }

        if let Some(target) = &self.explode {
            self.apply_explosions(&mut rng, roll, target, &mut exploded_rolls);
        }

        InitialDiceRollResult::new(
            self.group,
            self.sign,
            roll_number,
            self.dice_size,
            roll,
            discarded_rolls,
            exploded_rolls,
        )
    }

    fn apply_re_rolls(
        &self,
        rng: &mut impl Rng,
        initial_roll: u32,
        target: &Operator,
        discarded_rolls: &mut Vec<u32>,
    ) -> u32 {
        let mut roll = initial_roll;
        while apply_operator(*target, &roll) {
            discarded_rolls.push(roll);
            roll = rng.gen_range(1..=self.dice_size);
            if !self.re_roll_recursively {
                break;
            }
        }
        roll
    }

    fn apply_explosions(
        &self,
        rng: &mut impl Rng,
        initial_roll: u32,
        target: &Operator,
        exploded_rolls: &mut Vec<u32>,
    ) {
        if apply_operator(*target, &initial_roll) {
            let mut roll = rng.gen_range(1..=self.dice_size);
            exploded_rolls.push(roll);
            if !self.explode_once {
                while apply_operator(*target, &roll) {
                    roll = rng.gen_range(1..=self.dice_size);
                    exploded_rolls.push(roll);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct InitialDiceRollResult {
    pub group: i32,
    pub sign: i32,
    pub roll_number: u32,
    pub dice_size: u32,
    pub final_roll: u32,
    pub discarded_rolls: Vec<u32>,
    pub exploded_rolls: Vec<u32>,
    pub subtotal: i32,
}

impl InitialDiceRollResult {
    pub fn new(
        group: i32,
        sign: i32,
        roll_number: u32,
        dice_size: u32,
        final_roll: u32,
        discarded_rolls: Vec<u32>,
        exploded_rolls: Vec<u32>,
    ) -> InitialDiceRollResult {
        let subtotal = exploded_rolls.iter().map(|x| *x as i32).sum::<i32>() + final_roll as i32;
        Self {
            group,
            sign,
            roll_number,
            dice_size,
            final_roll,
            discarded_rolls,
            exploded_rolls,
            subtotal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Operator;

    #[test]
    fn test_basic_roll_no_reroll_no_explode() {
        let cmd = DiceRollCommand::new(1, 1, 3, 6, None, false, None, false);
        let results = cmd.roll_dice();
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.final_roll >= 1 && result.final_roll <= 6);
            assert!(result.discarded_rolls.is_empty());
            assert!(result.exploded_rolls.is_empty());
            assert_eq!(result.subtotal, result.final_roll as i32);
        }
    }

    #[test]
    fn test_reroll_once_on_1() {
        // Roll 1d6, reroll 1s once
        let cmd = DiceRollCommand::new(1, 1, 1, 6, Some(Operator::Eq(1)), false, None, false);
        let results = cmd.roll_dice();
        assert_eq!(results.len(), 1);
        let result = &results[0];

        // Either:
        // - first roll ≠ 1 → final_roll ≠ 1, no discarded
        // - first roll = 1 → final_roll ≠ 1, one discarded = 1
        if result.discarded_rolls.is_empty() {
            assert_ne!(result.final_roll, 1);
        } else {
            assert_eq!(result.discarded_rolls, vec![1]);
            assert_ne!(result.final_roll, 1);
        }
    }

    #[test]
    fn test_reroll_recursive_on_1() {
        let cmd = DiceRollCommand::new(1, 1, 1, 6, Some(Operator::Eq(1)), true, None, false);
        let results = cmd.roll_dice();
        let result = &results[0];

        // Final roll must not be 1
        assert_ne!(result.final_roll, 1);
        // All discarded rolls (if any) must be 1
        for &discarded in &result.discarded_rolls {
            assert_eq!(discarded, 1);
        }
    }

    #[test]
    fn test_explode_once_on_max() {
        let cmd = DiceRollCommand::new(1, 1, 1, 6, None, false, Some(Operator::Eq(6)), true);
        let results = cmd.roll_dice();
        let result = &results[0];

        if result.final_roll == 6 {
            // Should have exactly one exploded roll
            assert_eq!(result.exploded_rolls.len(), 1);
            let exploded = result.exploded_rolls[0];
            assert!(exploded >= 1 && exploded <= 6);
        } else {
            assert!(result.exploded_rolls.is_empty());
        }
        // Subtotal = final + sum(exploded)
        let expected_subtotal = result.final_roll as i32 + result.exploded_rolls.iter().map(|&x| x as i32).sum::<i32>();
        assert_eq!(result.subtotal, expected_subtotal);
    }

    #[test]
    fn test_explode_recursive_on_max() {
        let cmd = DiceRollCommand::new(1, 1, 1, 6, None, false, Some(Operator::Eq(6)), false);
        let results = cmd.roll_dice();
        let result = &results[0];

        // If final roll is 6, there must be at least one explosion
        // And no explosion roll should be 6 unless followed by another
        // But easier: just verify that no exploded roll triggers further explosion *unless* it's the last one
        // Instead, we check: all exploded rolls except possibly the last must be 6
        let exploded = &result.exploded_rolls;
        if result.final_roll == 6 {
            assert!(!exploded.is_empty());
            // All but the last exploded roll must be 6
            for &roll in exploded.iter().take(exploded.len().saturating_sub(1)) {
                assert_eq!(roll, 6);
            }
            // The last exploded roll can be anything (1–6)
            if let Some(&last) = exploded.last() {
                assert!(last >= 1 && last <= 6);
            }
        } else {
            assert!(exploded.is_empty());
        }

        let expected_subtotal = result.final_roll as i32 + exploded.iter().map(|&x| x as i32).sum::<i32>();
        assert_eq!(result.subtotal, expected_subtotal);
    }

    #[test]
    fn test_reroll_and_explode_together() {
        // Reroll 1s (once), explode on 6s (once)
        let cmd = DiceRollCommand::new(
            1, 1, 1, 6,
            Some(Operator::Eq(1)), false,
            Some(Operator::Eq(6)), true,
        );
        let results = cmd.roll_dice();
        let result = &results[0];

        // Final roll cannot be 1 (due to reroll)
        assert_ne!(result.final_roll, 1);

        // Discarded rolls (if any) must be [1]
        if !result.discarded_rolls.is_empty() {
            assert_eq!(result.discarded_rolls, vec![1]);
        }

        // Explode only if final_roll == 6
        if result.final_roll == 6 {
            assert_eq!(result.exploded_rolls.len(), 1);
        } else {
            assert!(result.exploded_rolls.is_empty());
        }

        let expected_subtotal = result.final_roll as i32 + result.exploded_rolls.iter().map(|&x| x as i32).sum::<i32>();
        assert_eq!(result.subtotal, expected_subtotal);
    }

    #[test]
    fn test_zero_dice_count() {
        let cmd = DiceRollCommand::new(1, 1, 0, 6, None, false, None, false);
        let results = cmd.roll_dice();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_dice_size_one() {
        // d1 always rolls 1
        let cmd = DiceRollCommand::new(1, 1, 2, 1, None, false, None, false);
        let results = cmd.roll_dice();
        assert_eq!(results.len(), 2);
        for r in results {
            assert_eq!(r.final_roll, 1);
            assert_eq!(r.subtotal, 1);
        }
    }

    #[test]
    fn test_reroll_always_condition() {
        // Reroll if <= 6 on d6 → always reroll, but non-recursive → only one reroll
        let cmd = DiceRollCommand::new(1, 1, 1, 6, Some(Operator::Lte(6)), false, None, false);
        let results = cmd.roll_dice();
        let result = &results[0];
        // Should have exactly one discarded roll (the first 1–6), and one final roll (also 1–6)
        assert_eq!(result.discarded_rolls.len(), 1);
        assert!(result.discarded_rolls[0] >= 1 && result.discarded_rolls[0] <= 6);
        assert!(result.final_roll >= 1 && result.final_roll <= 6);
    }
}