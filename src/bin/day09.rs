use std::{cmp, env, fs, io, io::Read};

use color_eyre::Result;
use fxhash::FxHashSet;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{map, value},
    sequence::{separated_pair, terminated},
    Finish, IResult,
};

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

    println!("{result:?}");

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Move {
    direction: Direction,
    distance: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    fn step(self, (x, y): (i32, i32)) -> (i32, i32) {
        match self {
            Direction::Up => (x, y + 1),
            Direction::Down => (x, y - 1),
            Direction::Right => (x + 1, y),
            Direction::Left => (x - 1, y),
        }
    }
}

fn step(s: &str) -> IResult<&str, Move> {
    map(
        terminated(
            separated_pair(dir, tag(" "), nom::character::complete::u8),
            line_ending,
        ),
        |(direction, distance)| Move {
            direction,
            distance,
        },
    )(s)
}

fn dir(s: &str) -> IResult<&str, Direction> {
    alt((
        value(Direction::Up, tag("U")),
        value(Direction::Down, tag("D")),
        value(Direction::Left, tag("L")),
        value(Direction::Right, tag("R")),
    ))(s)
}

fn run(mut input: &str) -> Result<(usize, usize)> {
    let mut snake2 = Snake::<2>::new();
    let mut snake10 = Snake::<10>::new();
    while let Ok((rem, step)) = step(input).map_err(|e| e.to_owned()).finish() {
        input = rem;
        for _ in 0..step.distance {
            snake2.advance(step.direction);
            snake10.advance(step.direction);
        }
        // println!("{:?}", snake10.segments);
    }

    Ok((snake2.tail_visits(), snake10.tail_visits()))
}

struct Snake<const N: usize> {
    segments: [(i32, i32); N],
    tail_positions: FxHashSet<(i32, i32)>,
}

impl<const N: usize> Snake<N> {
    fn new() -> Self {
        Self {
            segments: [(0, 0); N],
            tail_positions: [(0, 0)].into_iter().collect(),
        }
    }

    fn advance(&mut self, direction: Direction) {
        let prior_tail = self.tail_position();
        self.segments[0] = direction.step(self.segments[0]);
        let mut prior = self.segments[0];
        for seg in &mut self.segments[1..] {
            if !is_adjacent(*seg, prior) {
                seg.0 += catchup(prior.0, seg.0);
                seg.1 += catchup(prior.1, seg.1);
                prior = *seg;
            } else {
                break;
            }
        }

        if prior_tail != self.tail_position() {
            self.tail_positions.insert(self.tail_position());
        }
    }

    fn tail_position(&self) -> (i32, i32) {
        self.segments[N - 1]
    }

    fn tail_visits(&self) -> usize {
        self.tail_positions.len()
    }
}

fn is_adjacent((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> bool {
    (x1 - 1..=x1 + 1).contains(&x2) && (y1 - 1..=y1 + 1).contains(&y2)
}

fn catchup(h: i32, t: i32) -> i32 {
    match h.cmp(&t) {
        cmp::Ordering::Greater => 1,
        cmp::Ordering::Less => -1,
        cmp::Ordering::Equal => 0,
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day09test") => matches Ok((13, 1)))]
    #[test_case(include_str!("../../input/day09test2") => matches Ok((_, 36)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }
}
