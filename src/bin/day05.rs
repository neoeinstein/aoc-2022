use std::{
    io::{self, Read},
    iter,
};

use color_eyre::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res, value},
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};

fn main() -> Result<()> {
    let mut input = String::with_capacity(1024 * 8);
    io::stdin().read_to_string(&mut input)?;
    let (mut moves, mut ship) = ship(&input).map_err(|e| e.to_owned()).finish()?;
    let mut ship2 = ship.clone();
    loop {
        if moves.is_empty() {
            break;
        }
        let (rem, mv) = crate_move(moves).map_err(|e| e.to_owned()).finish()?;
        ship.move_crate(mv);
        ship2.move_several_crates(mv);
        moves = rem;
    }
    println!("Stack tops (9000): {}", ship.stack_tops());
    println!("Stack tops (9001): {}", ship2.stack_tops());
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SupplyCrate(char);

impl iter::Sum<SupplyCrate> for String {
    fn sum<I: Iterator<Item = SupplyCrate>>(iter: I) -> Self {
        let mut s = String::with_capacity(iter.size_hint().0);
        for crt in iter {
            s.push(crt.0);
        }
        s
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Stack(Vec<SupplyCrate>);

impl Stack {
    fn push(&mut self, crt: SupplyCrate) {
        self.0.push(crt);
    }

    fn pop(&mut self) -> SupplyCrate {
        self.0.pop().unwrap()
    }

    fn peek(&self) -> SupplyCrate {
        self.0.last().copied().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Ship(Vec<Stack>);

impl Ship {
    fn stack(&mut self, idx: usize) -> &mut Stack {
        &mut self.0[idx]
    }

    fn move_crate(&mut self, mv: Move) {
        for _ in 0..mv.count {
            let crt = self.stack(mv.source - 1).pop();
            self.stack(mv.destination - 1).push(crt);
        }
    }

    fn move_several_crates(&mut self, mv: Move) {
        if mv.source == mv.destination {
            return;
        }
        let first = mv.source.min(mv.destination) - 1;
        let second = mv.source.max(mv.destination) - first - 1;
        let (_rem, left) = self.0.split_at_mut(first);
        let (left, right) = left.split_at_mut(second);
        // println!("({}, {}, {})", rem.len(), left.len(), right.len());
        let (origin, dest) = if mv.source < mv.destination {
            (&mut left[0].0, &mut right[0].0)
        } else {
            (&mut right[0].0, &mut left[0].0)
        };
        dest.extend(origin.drain((origin.len() - mv.count)..));
    }

    fn stack_tops(&self) -> String {
        self.0.iter().map(|stack| stack.peek()).sum()
    }
}

fn supply_crate(s: &str) -> IResult<&str, SupplyCrate> {
    map(
        delimited(tag("["), nom::character::complete::anychar, tag("]")),
        SupplyCrate,
    )(s)
}

fn maybe_crate(s: &str) -> IResult<&str, Option<SupplyCrate>> {
    alt((map(supply_crate, Some), value(None, tag("   "))))(s)
}

fn crate_row(s: &str) -> IResult<&str, Vec<Option<SupplyCrate>>> {
    separated_list1(tag(" "), maybe_crate)(s)
}

fn labels_row(s: &str) -> IResult<&str, usize> {
    map(
        delimited(tag(" "), separated_list1(tag("   "), digit1), tag(" ")),
        |l| l.len(),
    )(s)
}

fn ship(s: &str) -> IResult<&str, Ship> {
    map(
        terminated(
            separated_pair(separated_list1(tag("\n"), crate_row), tag("\n"), labels_row),
            tag("\n\n"),
        ),
        |(rows, stacks)| {
            let mut ship = Ship(vec![Stack::default(); stacks]);
            for row in rows.into_iter().rev() {
                for (idx, crt) in row.into_iter().enumerate() {
                    if let Some(crt) = crt {
                        ship.stack(idx).push(crt);
                    }
                }
            }
            ship
        },
    )(s)
}

#[derive(Clone, Copy, Debug)]
struct Move {
    count: usize,
    source: usize,
    destination: usize,
}

fn crate_move(s: &str) -> IResult<&str, Move> {
    terminated(
        map(
            tuple((
                preceded(tag("move "), map_res(digit1, str::parse)),
                preceded(tag(" from "), map_res(digit1, str::parse)),
                preceded(tag(" to "), map_res(digit1, str::parse)),
            )),
            |(count, source, destination)| Move {
                count,
                source,
                destination,
            },
        ),
        tag("\n"),
    )(s)
}
