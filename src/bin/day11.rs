use std::{
    cell::{Cell, RefCell},
    env, fs, io,
    io::Read,
};

use color_eyre::Result;
// use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, line_ending},
    combinator::{all_consuming, map, value},
    error::{convert_error, VerboseError},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    Finish, IResult,
};
use num::Integer;

fn main() -> Result<()> {
    color_eyre::install()?;

    let input = if let Some(path) = env::args_os().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    };

    let result = run(&input)?;

    println!("Part 1: \n{:?}\n\nPart 2: \n{:?}", result.0, result.1);

    Ok(())
}

const NO_CALM: i64 = 1;
const SOME_CALM: i64 = 3;

#[inline(never)]
fn run(input: &str) -> Result<(usize, usize)> {
    let mut monkeys = Monkeys::parse_complete(input)?;
    let worry_modulus = monkeys.worry_modulus();
    monkeys.calming = Calming::new(SOME_CALM, worry_modulus);

    let mut monkeys_part2 = monkeys.clone();
    monkeys_part2.calming = Calming::new(NO_CALM, worry_modulus);

    for _round in 1..=20 {
        // println!("Round {}", _round);
        monkeys.execute_round();
    }

    for _round in 1..=10000 {
        // println!("Round {}", round);
        monkeys_part2.execute_round();
    }

    Ok((monkeys.monkey_business(), monkeys_part2.monkey_business()))
}

#[derive(Clone, Debug)]
struct Monkeys {
    monkeys: Vec<Monkey>,
    calming: Calming,
}

impl Monkeys {
    #[inline(never)]
    fn parse_complete(input: &str) -> Result<Self> {
        all_consuming(Self::parse)(input)
            .finish()
            .map(|o| o.1)
            .map_err(|e| color_eyre::Report::msg(convert_error(input, e)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Calming {
    Division(i64),
    Modular {
        modulus: i64,
        // multiplicative_mod_inverse: i64,
    },
}

impl Default for Calming {
    fn default() -> Self {
        Calming::Division(1)
    }
}

impl Calming {
    fn new(calming_factor: i64, modulus: i64) -> Self {
        if calming_factor != 1 {
            Self::Division(calming_factor)
        } else {
            // let gcd = dbg!(calming_factor.extended_gcd(&modulus));
            // let mut multiplicative_mod_inverse = gcd.x;
            // multiplicative_mod_inverse %= modulus;
            // multiplicative_mod_inverse += modulus;
            // multiplicative_mod_inverse %= modulus;
            Self::Modular {
                modulus,
                // multiplicative_mod_inverse,
            }
        }
    }

    fn calm(&self, worry: &mut i64) {
        match self {
            Self::Division(d) => *worry /= d,
            Self::Modular {
                modulus,
                // multiplicative_mod_inverse,
            } => {
                *worry %= modulus;
                // *worry *= multiplicative_mod_inverse;
                // *worry %= modulus;
            }
        }
    }
}

impl Monkeys {
    fn parse(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
        map(separated_list0(line_ending, Monkey::parse), |monkeys| {
            Monkeys {
                monkeys,
                calming: Calming::default(),
            }
        })(s)
    }

    fn worry_modulus(&self) -> i64 {
        self.monkeys
            .iter()
            .map(|m| m.test.divisor)
            .fold(1, |acc, d| acc.lcm(&d))
    }

    fn execute_round(&self) {
        for (_idx, monkey) in self.monkeys.iter().enumerate() {
            // println!("Monkey {}:", idx);
            for (item, target) in monkey.take_turn(self.calming) {
                // println!("  {:?} -> {}", item, target);
                self.monkeys[target].catch(item);
            }
        }
        // for (idx, monkey) in self.monkeys.iter().enumerate() {
        //     println!("Monkey {}: {}", idx, monkey.items.borrow().iter().map(|i|
        // i.worry.to_string()).join(", ")); }
        // println!();
    }

    fn monkey_business(&self) -> usize {
        // for monkey in &self.monkeys {
        //     println!("{}", monkey.inspected.get());
        // }
        let val = self.monkeys.iter().fold((0, 0), |acc, m| {
            let inspected = m.inspected.get();
            if inspected > acc.0 {
                (inspected, acc.0)
            } else if inspected > acc.1 {
                (acc.0, inspected)
            } else {
                acc
            }
        });
        val.0 * val.1
    }
}

#[derive(Clone, Debug)]
struct Monkey {
    items: RefCell<Vec<Item>>,
    worry_op: WorryOp,
    test: Test,
    inspected: Cell<usize>,
}

impl Monkey {
    fn parse(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
        preceded(
            delimited(tag("Monkey "), digit1, tuple((tag(":"), line_ending))),
            map(
                tuple((
                    delimited(
                        tag("  Starting items: "),
                        separated_list0(tag(", "), Item::parse),
                        line_ending,
                    ),
                    WorryOp::parse,
                    Test::parse,
                )),
                |(items, worry_op, test)| Monkey {
                    items: RefCell::new(items),
                    worry_op,
                    test,
                    inspected: Cell::new(0),
                },
            ),
        )(s)
    }

    fn take_turn(&self, calming: Calming) -> impl Iterator<Item = (Item, usize)> {
        let items = std::mem::take(&mut *self.items.borrow_mut());
        self.inspected.replace(self.inspected.get() + items.len());
        let worry_op = self.worry_op;
        let test = self.test;
        items.into_iter().map(move |mut item| {
            match worry_op {
                WorryOp::Additive(n) => item.worry += n,
                WorryOp::Multiplicative(n) => item.worry *= n,
                WorryOp::Squared => item.worry *= item.worry,
            };
            calming.calm(&mut item.worry);
            let target = test.apply(&item);
            (item, target)
        })
    }

    fn catch(&self, item: Item) {
        self.items.borrow_mut().push(item);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Item {
    worry: i64,
}

impl Item {
    fn parse(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
        map(nom::character::complete::i64, |worry| Item { worry })(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorryOp {
    Additive(i64),
    Multiplicative(i64),
    Squared,
}

impl WorryOp {
    fn parse(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
        delimited(
            tag("  Operation: new = old "),
            alt((
                value(WorryOp::Squared, tag("* old")),
                map(
                    preceded(tag("+ "), nom::character::complete::i64),
                    WorryOp::Additive,
                ),
                map(
                    preceded(tag("* "), nom::character::complete::i64),
                    WorryOp::Multiplicative,
                ),
            )),
            line_ending,
        )(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Test {
    divisor: i64,
    true_monkey: usize,
    false_monkey: usize,
}

impl Test {
    fn parse(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
        map(
            tuple((
                delimited(
                    tag("  Test: divisible by "),
                    nom::character::complete::i64,
                    line_ending,
                ),
                delimited(
                    tag("    If true: throw to monkey "),
                    nom::character::complete::u32,
                    line_ending,
                ),
                delimited(
                    tag("    If false: throw to monkey "),
                    nom::character::complete::u32,
                    line_ending,
                ),
            )),
            |(divisor, true_monkey, false_monkey)| Test {
                divisor,
                true_monkey: true_monkey as usize,
                false_monkey: false_monkey as usize,
            },
        )(s)
    }

    fn apply(&self, item: &Item) -> usize {
        if item.worry % self.divisor == 0 {
            self.true_monkey
        } else {
            self.false_monkey
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day11test") => matches Ok((10605, 2713310158)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }
}
