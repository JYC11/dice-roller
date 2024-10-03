use crate::enums::Operator;
use rand::Rng;
#[derive(Copy, Clone, Debug)]
pub struct DiceRollCommand {
    group: i32,
    sign: i32,
    dice_count: u32,
    dice_size: u32,
    re_roll: Option<Operator>,
    re_roll_recursively: bool,
    explode: Option<Operator>,
    explode_once: bool,
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
        let mut rng = rand::thread_rng();
        let mut current_dice_count = 0;
        let mut individual_results: Vec<InitialDiceRollResult> = vec![];

        while current_dice_count != self.dice_count {
            let mut discarded_rolls: Vec<u32> = vec![];
            let mut exploded_rolls: Vec<u32> = vec![];

            let roll_result = rng.gen_range(1..self.dice_size + 1);
            let mut final_roll: u32 = roll_result;
            match self.re_roll {
                None => final_roll = roll_result,
                Some(target) => {
                    let needs_re_roll = self.needs_re_roll(&roll_result, &target);
                    if needs_re_roll && !self.re_roll_recursively {
                        let second_result = rng.gen_range(1..self.dice_size + 1);
                        discarded_rolls.push(roll_result);
                        final_roll = second_result;
                    } else if needs_re_roll && self.re_roll_recursively {
                        let mut new_roll = rng.gen_range(1..self.dice_size + 1);
                        while self.needs_re_roll(&new_roll, &target) {
                            discarded_rolls.push(new_roll);
                            new_roll = rng.gen_range(1..self.dice_size + 1);
                        }
                        final_roll = new_roll;
                    } else if !needs_re_roll {
                        final_roll = roll_result;
                    }
                }
            }

            match self.explode {
                None => {}
                Some(target) => {
                    let needs_explosion = self.needs_explosion(&final_roll, &target);
                    if needs_explosion && self.explode_once {
                        exploded_rolls.push(rng.gen_range(1..self.dice_size + 1));
                    }
                    if needs_explosion && !self.explode_once {
                        let mut exploded_roll = rng.gen_range(1..self.dice_size + 1);
                        exploded_rolls.push(exploded_roll);
                        while self.needs_explosion(&exploded_roll, &target) {
                            exploded_roll = rng.gen_range(1..self.dice_size + 1);
                            exploded_rolls.push(exploded_roll);
                        }
                    }
                    if !needs_explosion {}
                }
            }

            individual_results.push(InitialDiceRollResult::new(
                self.group,
                self.sign,
                current_dice_count + 1,
                self.dice_size,
                final_roll,
                discarded_rolls,
                exploded_rolls,
            ));
            current_dice_count += 1
        }

        individual_results
    }

    fn needs_re_roll(&self, source: &u32, target: &Operator) -> bool {
        match target {
            Operator::Eq(target) => source == target,
            Operator::Gt(target) => source > target,
            Operator::Gte(target) => source >= target,
            Operator::Lt(target) => source < target,
            Operator::Lte(target) => source <= target,
        }
    }

    fn needs_explosion(&self, source: &u32, target: &Operator) -> bool {
        match target {
            Operator::Eq(target) => source == target,
            Operator::Gt(target) => source > target,
            Operator::Gte(target) => source >= target,
            Operator::Lt(target) => source < target,
            Operator::Lte(target) => source <= target,
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
