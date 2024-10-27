use crate::enums::Operator;
use rand::Rng;
use crate::utils::apply_operator;

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
